#!/usr/bin/env python3
"""
MViz ROS2 Bridge - Dora Operator

Bridges ROS2 topics to Dora dataflow for visualization in MViz.
Subscribes to common ROS2 message types and converts them to Float32Array
format compatible with mviz-bridge.

ROS2 Message Type Mappings:
    sensor_msgs/PointCloud2  -> points3d (xyz float array)
    geometry_msgs/PoseStamped -> transform3d (x,y,z,qx,qy,qz,qw)
    nav_msgs/Odometry        -> odom_pose (x,y,z,qx,qy,qz,qw)
    nav_msgs/Path            -> waypoints (x1,y1,x2,y2,...)
    sensor_msgs/Imu          -> imu_msg (qx,qy,qz,qw,gx,gy,gz,ax,ay,az)
    visualization_msgs/Marker -> boxes3d/arrows3d (depends on marker type)

Configuration:
    Edit the TOPIC_MAPPINGS dict below to configure which ROS2 topics to bridge.

Usage:
    This script is used as a Dora operator. See dataflow-ros2.yml.
"""

import os
import numpy as np
import pyarrow as pa

# Check if ROS2 is available
try:
    import rclpy
    from rclpy.node import Node
    from rclpy.qos import QoSProfile, ReliabilityPolicy, HistoryPolicy
    ROS2_AVAILABLE = True
except ImportError:
    ROS2_AVAILABLE = False
    print("[ros2_bridge] WARNING: rclpy not found. Install ROS2 or source setup.bash")

# Import ROS2 message types (only if ROS2 available)
if ROS2_AVAILABLE:
    try:
        from sensor_msgs.msg import PointCloud2, Imu, Image
        from geometry_msgs.msg import PoseStamped, Pose
        from nav_msgs.msg import Odometry, Path
        from visualization_msgs.msg import Marker, MarkerArray
        import sensor_msgs_py.point_cloud2 as pc2
    except ImportError as e:
        print(f"[ros2_bridge] WARNING: Some ROS2 message types not found: {e}")


# =============================================================================
# CONFIGURATION - Edit these mappings for your ROS2 system
# =============================================================================

TOPIC_MAPPINGS = {
    # ROS2 Topic -> (Dora Output ID, Message Type)
    # Uncomment/modify the topics you want to bridge

    # Point clouds
    "/velodyne_points": ("pointcloud", "PointCloud2"),
    "/lidar/points": ("pointcloud", "PointCloud2"),
    "/camera/depth/points": ("pointcloud", "PointCloud2"),

    # Poses and odometry
    "/robot_pose": ("pose", "PoseStamped"),
    "/odom": ("odom_pose", "Odometry"),
    "/amcl_pose": ("pose", "PoseStamped"),

    # Paths and waypoints
    "/planned_path": ("waypoints", "Path"),
    "/global_plan": ("waypoints", "Path"),
    "/move_base/NavfnROS/plan": ("waypoints", "Path"),

    # IMU
    "/imu/data": ("imu_msg", "Imu"),
    "/imu": ("imu_msg", "Imu"),

    # Markers (for visualization_msgs)
    "/visualization_marker": ("markers", "Marker"),
    "/visualization_marker_array": ("markers", "MarkerArray"),
}

# QoS profile for sensor data
SENSOR_QOS = None
if ROS2_AVAILABLE:
    SENSOR_QOS = QoSProfile(
        reliability=ReliabilityPolicy.BEST_EFFORT,
        history=HistoryPolicy.KEEP_LAST,
        depth=1
    )


# =============================================================================
# Message Conversion Functions
# =============================================================================

def pointcloud2_to_float32(msg) -> np.ndarray:
    """Convert sensor_msgs/PointCloud2 to Float32Array [x,y,z,x,y,z,...]"""
    try:
        points = list(pc2.read_points(msg, field_names=("x", "y", "z"), skip_nans=True))
        if not points:
            return np.array([], dtype=np.float32)
        arr = np.array(points, dtype=np.float32).flatten()
        return arr
    except Exception as e:
        print(f"[ros2_bridge] PointCloud2 conversion error: {e}")
        return np.array([], dtype=np.float32)


def pose_to_float32(pose: Pose) -> np.ndarray:
    """Convert geometry_msgs/Pose to Float32Array [x,y,z,qx,qy,qz,qw]"""
    return np.array([
        pose.position.x,
        pose.position.y,
        pose.position.z,
        pose.orientation.x,
        pose.orientation.y,
        pose.orientation.z,
        pose.orientation.w,
    ], dtype=np.float32)


def posestamped_to_float32(msg) -> np.ndarray:
    """Convert geometry_msgs/PoseStamped to Float32Array"""
    return pose_to_float32(msg.pose)


def odometry_to_float32(msg) -> np.ndarray:
    """Convert nav_msgs/Odometry to Float32Array [x,y,z,qx,qy,qz,qw]"""
    return pose_to_float32(msg.pose.pose)


def path_to_float32(msg) -> np.ndarray:
    """Convert nav_msgs/Path to Float32Array [x1,y1,x2,y2,...]"""
    if not msg.poses:
        return np.array([], dtype=np.float32)

    points = []
    for pose_stamped in msg.poses:
        points.extend([
            pose_stamped.pose.position.x,
            pose_stamped.pose.position.y,
        ])
    return np.array(points, dtype=np.float32)


def imu_to_float32(msg) -> np.ndarray:
    """Convert sensor_msgs/Imu to Float32Array [qx,qy,qz,qw,gx,gy,gz,ax,ay,az]"""
    return np.array([
        msg.orientation.x,
        msg.orientation.y,
        msg.orientation.z,
        msg.orientation.w,
        msg.angular_velocity.x,
        msg.angular_velocity.y,
        msg.angular_velocity.z,
        msg.linear_acceleration.x,
        msg.linear_acceleration.y,
        msg.linear_acceleration.z,
    ], dtype=np.float32)


def marker_to_float32(msg) -> np.ndarray:
    """Convert visualization_msgs/Marker to Float32Array (format depends on type)"""
    # Marker types: ARROW=0, CUBE=1, SPHERE=2, CYLINDER=3, LINE_STRIP=4, etc.
    if msg.type == Marker.CUBE or msg.type == Marker.SPHERE:
        # Return as box: [x,y,z,sx,sy,sz,qx,qy,qz,qw]
        return np.array([
            msg.pose.position.x,
            msg.pose.position.y,
            msg.pose.position.z,
            msg.scale.x,
            msg.scale.y,
            msg.scale.z,
            msg.pose.orientation.x,
            msg.pose.orientation.y,
            msg.pose.orientation.z,
            msg.pose.orientation.w,
        ], dtype=np.float32)
    elif msg.type == Marker.ARROW:
        # Return as arrow: origin + vector
        # If points are specified, use them; otherwise use pose
        if len(msg.points) >= 2:
            return np.array([
                msg.points[0].x, msg.points[0].y, msg.points[0].z,
                msg.points[1].x - msg.points[0].x,
                msg.points[1].y - msg.points[0].y,
                msg.points[1].z - msg.points[0].z,
            ], dtype=np.float32)
        else:
            return np.array([
                msg.pose.position.x,
                msg.pose.position.y,
                msg.pose.position.z,
                msg.scale.x,  # Arrow length in x direction
                0.0,
                0.0,
            ], dtype=np.float32)
    elif msg.type == Marker.LINE_STRIP or msg.type == Marker.LINE_LIST:
        # Return as line strip: [x1,y1,z1,x2,y2,z2,...]
        points = []
        for p in msg.points:
            points.extend([p.x, p.y, p.z])
        return np.array(points, dtype=np.float32)
    else:
        # Default: just position
        return np.array([
            msg.pose.position.x,
            msg.pose.position.y,
            msg.pose.position.z,
        ], dtype=np.float32)


# Conversion function lookup
CONVERTERS = {
    "PointCloud2": pointcloud2_to_float32,
    "PoseStamped": posestamped_to_float32,
    "Odometry": odometry_to_float32,
    "Path": path_to_float32,
    "Imu": imu_to_float32,
    "Marker": marker_to_float32,
}

# ROS2 message type lookup
MSG_TYPES = {}
if ROS2_AVAILABLE:
    MSG_TYPES = {
        "PointCloud2": PointCloud2,
        "PoseStamped": PoseStamped,
        "Odometry": Odometry,
        "Path": Path,
        "Imu": Imu,
        "Marker": Marker,
        "MarkerArray": MarkerArray,
    }


# =============================================================================
# ROS2 Bridge Node
# =============================================================================

class Ros2BridgeNode(Node):
    """ROS2 node that subscribes to topics and stores latest messages."""

    def __init__(self):
        super().__init__('mviz_ros2_bridge')
        self.latest_data = {}  # output_id -> numpy array
        self.subscriptions_list = []

        self.get_logger().info("Initializing MViz ROS2 Bridge")
        self.get_logger().info(f"Configured topics: {list(TOPIC_MAPPINGS.keys())}")

        # Create subscriptions for each configured topic
        for topic, (output_id, msg_type_name) in TOPIC_MAPPINGS.items():
            if msg_type_name not in MSG_TYPES:
                self.get_logger().warn(f"Unknown message type: {msg_type_name} for topic {topic}")
                continue

            msg_type = MSG_TYPES[msg_type_name]
            converter = CONVERTERS.get(msg_type_name)

            if converter is None:
                self.get_logger().warn(f"No converter for message type: {msg_type_name}")
                continue

            # Create callback closure
            def make_callback(out_id, conv, topic_name):
                def callback(msg):
                    try:
                        data = conv(msg)
                        if len(data) > 0:
                            self.latest_data[out_id] = data
                            self.get_logger().debug(
                                f"Received {topic_name} -> {out_id}: {len(data)} floats"
                            )
                    except Exception as e:
                        self.get_logger().error(f"Error processing {topic_name}: {e}")
                return callback

            # Use sensor QoS for sensor data
            qos = SENSOR_QOS if msg_type_name in ["PointCloud2", "Imu", "Image"] else 10

            sub = self.create_subscription(
                msg_type,
                topic,
                make_callback(output_id, converter, topic),
                qos
            )
            self.subscriptions_list.append(sub)
            self.get_logger().info(f"Subscribed to {topic} -> {output_id}")

    def get_and_clear(self, output_id: str) -> np.ndarray:
        """Get latest data for output_id and clear it."""
        return self.latest_data.pop(output_id, None)


# =============================================================================
# Dora Operator
# =============================================================================

# Global ROS2 node (initialized once)
_ros2_node = None
_ros2_initialized = False


class Operator:
    """Dora operator that bridges ROS2 topics to Dora outputs."""

    def __init__(self):
        global _ros2_node, _ros2_initialized

        if not ROS2_AVAILABLE:
            print("[ros2_bridge] ROS2 not available - running in dummy mode")
            self.dummy_mode = True
            return

        self.dummy_mode = False

        # Initialize ROS2 (only once)
        if not _ros2_initialized:
            print("[ros2_bridge] Initializing ROS2...")
            rclpy.init()
            _ros2_initialized = True

        # Create ROS2 node (only once)
        if _ros2_node is None:
            _ros2_node = Ros2BridgeNode()
            print("[ros2_bridge] ROS2 bridge node created")

        self.node = _ros2_node
        print("[ros2_bridge] Operator initialized")

    def on_event(self, dora_event, send_output):
        """Handle Dora events - spin ROS2 and forward data."""
        from dora import DoraStatus

        if dora_event["type"] == "INPUT" and dora_event["id"] == "tick":
            if self.dummy_mode:
                # In dummy mode, just continue
                return DoraStatus.CONTINUE

            # Spin ROS2 to process callbacks (non-blocking)
            rclpy.spin_once(self.node, timeout_sec=0.001)

            # Check for new data and send to Dora outputs
            for output_id in ["pointcloud", "pose", "odom_pose", "waypoints", "imu_msg", "markers"]:
                data = self.node.get_and_clear(output_id)
                if data is not None and len(data) > 0:
                    # Convert to PyArrow array for Dora
                    arrow_array = pa.array(data.tolist(), type=pa.float32())
                    send_output(output_id, arrow_array)

        elif dora_event["type"] == "STOP":
            print("[ros2_bridge] Shutting down...")
            return DoraStatus.STOP

        return DoraStatus.CONTINUE


def main():
    """Test ROS2 bridge standalone (without Dora)."""
    if not ROS2_AVAILABLE:
        print("ROS2 not available. Please source ROS2 setup.bash")
        return

    print("Testing ROS2 Bridge standalone...")
    rclpy.init()
    node = Ros2BridgeNode()

    try:
        while rclpy.ok():
            rclpy.spin_once(node, timeout_sec=0.1)

            # Print any received data
            for output_id, data in list(node.latest_data.items()):
                print(f"  {output_id}: {len(data)} floats - {data[:6]}...")
                del node.latest_data[output_id]
    except KeyboardInterrupt:
        pass
    finally:
        node.destroy_node()
        rclpy.shutdown()


if __name__ == "__main__":
    main()

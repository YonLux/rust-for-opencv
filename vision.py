import cv2
import socket
from cvzone.HandTrackingModule import HandDetector

# UDP 配置
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
addr = ("127.0.0.1", 9999)

# 摄像头配置
cap = cv2.VideoCapture(0)
cap.set(3, 1280)  # 宽
cap.set(4, 720)   # 高

detector = HandDetector(detectionCon=0.8, maxHands=1)

print("Vision Controller Started...")

while True:
    success, img = cap.read()
    if not success or img is None:
        continue

    # 镜像处理，方便操作
    img = cv2.flip(img, 1)
    
    # 手势检测
    hands, img = detector.findHands(img, flipType=False)

    if hands:
        # 获取食指指尖坐标 (Landmark 8)
        lmList = hands[0]["lmList"]
        pointIndex = lmList[8][0:2] # 获取 x, y
        
        # 发送坐标到 Rust
        msg = f"{pointIndex[0]},{pointIndex[1]}"
        sock.sendto(msg.encode(), addr)
        
        # 画个圈提示正在追踪
        cv2.circle(img, (pointIndex[0], pointIndex[1]), 15, (0, 255, 255), cv2.FILLED)

    cv2.imshow("Vision Controller (Hand Tracker)", img)
    
    if cv2.waitKey(1) == 27:
        break

cap.release()
cv2.destroyAllWindows()
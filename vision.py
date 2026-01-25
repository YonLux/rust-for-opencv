import cv2
import socket
from cvzone.HandTrackingModule import HandDetector

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
addr = ("127.0.0.1", 9999)

cap = cv2.VideoCapture(0)
detector = HandDetector(maxHands=1)

while True:
    _, img = cap.read()
    img = cv2.flip(img, 1)
    hands, img = detector.findHands(img)

    if hands:
        lm = hands[0]["lmList"]
        x, y = lm[8][0], lm[8][1]
        sock.sendto(f"{x},{y}".encode(), addr)

    cv2.imshow("Vision", img)
    if cv2.waitKey(1) == 27:
        break



"""
[Python摄像头视觉]  ---> UDP ---> [Rust贪吃蛇游戏]   相互通信

"""
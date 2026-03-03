import cv2 as cv
import numpy as np


def findpoint(img):
    kernel_size = 5
    kernel = np.ones((kernel_size, kernel_size), np.uint8)


    img = cv.imread('test.png')
    gray = cv.cvtColor(img,cv.COLOR_BGR2GRAY)
    gray = np.float32(gray)

    dst = cv.cornerHarris(gray,2,3,0.04)
    dst_dilated = cv.dilate(dst, kernel)
    # img[dst>0.01*dst.max()]=[0,0,255]
    threshold = 0.01 * dst.max()
    #局部非极大值抑制
    corner_mask = (dst == dst_dilated) & (dst > threshold)
    y_indices, x_indices = np.where(corner_mask)
    corner_coords = list(zip(x_indices, y_indices))

    return corner_coords

def calculate(corner_coords):
    pass
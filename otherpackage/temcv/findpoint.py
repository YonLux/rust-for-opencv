import cv2 as cv
import numpy as np
from matplotlib import pyplot as plt
from collections import defaultdict
kernel_size = 5
kernel = np.ones((kernel_size, kernel_size), np.uint8)
error_calcu= 200
sys_test =0
# TODO this code have some bug:cant detect completly i do not konw why
"""
03032026 so how to solve 

"""
if sys_test == 0:
    img = cv.imread('test.png')
elif sys_test == 1:
    img = cv.imread('test1.png')
elif sys_test == 2:
    img = cv.imread('test2.png')

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
print(corner_coords)
for pt in corner_coords:
    x, y = pt
    cv.circle(img, (x, y), radius=6, color=(255, 0, 255), thickness=-1)
    cv.putText(img, f"{x},{y}", (x + 20, y - 20), 
        cv.FONT_HERSHEY_SIMPLEX, 0.5, (125, 125, 0), 2)

#print(corner_coords[0][0])
#exit(0)
point_list= []
mid = []
tangle = []
for s,v in enumerate(corner_coords):
    # print(s)
    for j in range(s+1):
        if s == j:
            continue
        else:
            x_mid_point =(corner_coords[s][0] + corner_coords[j][0])//2
            y_mid_point =(corner_coords[s][1] + corner_coords[j][1])//2
            #mid_point=(x_mid_point,y_mid_pointj
            #print(mid_point)
            longth = (corner_coords[s][0] - corner_coords[j][0])**2 + (corner_coords[s][1] - corner_coords[j][1])**2 
            point = (x_mid_point,y_mid_point,longth,((corner_coords[s][0],corner_coords[s][1]),(corner_coords[j][0],corner_coords[j][1])))
            point_list.append(point)
print("-----------------------testsettestestestestest-------------------------------")
print(point_list[12],point_list[9])

# hash
for i,v in enumerate(point_list):
    for j in range(i+1):
        if i == j:
            continue
        else:
            x1= point_list[j][3][0][0]
            y1= point_list[j][3][0][1]

            x1_f= point_list[j][3][1][0]
            y1_f= point_list[j][3][1][1]  

            x2= point_list[i][3][0][0]
            y2= point_list[i][3][0][1]

            x2_f= point_list[i][3][1][0]
            y2_f= point_list[i][3][1][1]              
                      
            contact_side_len_0 = (x1 - x2)**2 + (y1 - y2)**2 
            contact_side_len_1 = (x1_f - x2_f)**2 + (y1_f-y2_f)**2 
            # contact_side_len_0 = (x1 - x2)**2 + (y1 - y2)**2 
            # contact_side_len_1 = (x1_f - x2_f)**2 + (y1_f-y2_f)**2 

            #rate = (contact_side_len_1 - contact_side_len_0)/(contact_side_len_1 + contact_side_len_0)
            rate = (contact_side_len_1 - contact_side_len_0)
            if (-error_calcu <= point_list[i][0] - point_list[j][0] <= error_calcu) and (-error_calcu <= point_list[i][1] - point_list[j][1] <= error_calcu) and -20<=rate<=20:
                mid.append((i,j))  
                print("mid%d",mid)

                

for i,v in enumerate(mid):
    print(i)
    
    #tan = (point_list[v[0]][3][0],point_list[v[0]][3][1],point_list[v[1]][3][0],point_list[v[1]][3][1])
    tan = (point_list[v[0]][3][0],point_list[v[1]][3][0],point_list[v[0]][3][1],point_list[v[1]][3][1])
    print(tan)
    #print(v[0],v[1])
    a1_x = tan[0][0] - tan[3][0]
    a1_y = tan[0][1] - tan[3][1]

    a2_x = tan[0][0] - tan[1][0]
    a2_y = tan[0][1] - tan[1][1]
    #print("value:%d",(a1_x * a2_x) + (a1_y * a2_y))

    if -5000<((a1_x * a2_x) + (a1_y * a2_y))<5000:
    #if 1:    
        tangle.append(tan)
        #print("value:%d",((a1_x * a2_x) + (a1_y * a2_y)))
    else:
        continue

#exit(0)


points  = tangle[0]
for pt in points:
    x, y = pt
    cv.circle(img, (x, y), radius=6, color=(0, 0, 255), thickness=-1)
    cv.putText(img, f"{x},{y}", (x + 10, y - 10), 
            cv.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 1)
if len(tangle)>1:    
    points  = tangle[1]
    for pt in points:
        x, y = pt
        cv.circle(img, (x, y), radius=6, color=(0, 225, 255), thickness=-1)
        cv.putText(img, f"{x},{y}", (x + 10, y - 10), 
                cv.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 1)

# print(point_list)
# print("===================tan::")
# print(tangle)
print("===================mid")
print(tangle)
#print(len(tangle))
#exit(0)
if(1):
    for i,v in enumerate(tangle):
        #print(tangle[i])
        pts = np.array(tangle[i], dtype=np.int32)
        pts = pts.reshape((-1, 1, 2))
        cv.polylines(img, [pts], isClosed=True, color=(0, 255, 0), thickness=2)
    cv.imshow('dst',img)



if(0):
    if len(corner_coords) > 0:
        print("前 5 个角点的 (x, y):", corner_coords[:8])
        print(len(corner_coords))
        print(dst.max())
    for a in corner_coords:
        cv.circle(img,a,5,(0,255,255))
    cv.imshow('dst',img)

    #cv.imshow('dst_dilated',dst_dilated)
    cv.imshow('dst',img)
    #cv.imshow('mask',dst)
# plt.imshow(img)



if cv.waitKey(0) & 0xff == 27:
    cv.destroyAllWindows()
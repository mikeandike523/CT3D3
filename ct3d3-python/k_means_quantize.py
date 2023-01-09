import numpy as np
import cv2

def k_means_quantize(img, K=2):

    img = img.copy().astype(np.float32)
    EPS =0.05
    MAX_ITER = 50
    MAX_TRIES = 25
    data = img.reshape((-1, 1))
    criteria =  (
        cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER,
        MAX_ITER,
        EPS
    )
    compactness, labels, centers = cv2.kmeans(data, K, None, criteria, MAX_TRIES, cv2.KMEANS_RANDOM_CENTERS)
    labels = labels.astype(int).squeeze().reshape(img.shape)
    centers = centers.squeeze()


    sortorder = np.argsort(centers)
    inverse_sortorder = [list(sortorder).index(i) for i in range(len(sortorder))]
    inverse_sortorder = np.array(inverse_sortorder, int)

    centers = centers[sortorder]
    labels = inverse_sortorder[labels]
    qimg = centers[labels]

    return centers, labels, qimg









# --- Attributions
# Finding dicom image plane with python: https://stackoverflow.com/a/56670334/5166365
# ---

from pydicom import dcmread
import numpy as np
import os
import struct
import argparse
from natsort import natsorted

parser = argparse.ArgumentParser()
parser.add_argument("--dropped-file")
args = parser.parse_args()

DICOM_DIR = os.path.dirname(args.dropped_file)

AXIAL = "axial"
SAGITAL = "sagital"
CORONAL = "coronal"

AXIAL_COSINES = (1,0,0,0,1,0)
SAGITAL_COSINES =  (0,1,0,0,0,-1)
CORONAL_COSINES = (1,0,0,0,0,-1)

def rescale_array(array,minimum_range=0.000010):
    min_val=np.amin(array)
    max_val=np.amax(array)
    if max_val-min_val >= minimum_range:
        return (array.copy()-min_val)/(max_val-min_val)
    return None

class DicomFile:
    
    @classmethod
    def direction_cosines_to_plane(cls,direction_cosines):

        direction_cosines = list(map(int,direction_cosines))
        direction_cosines = list(map(abs,direction_cosines))
        direction_cosines = tuple(direction_cosines)

        if direction_cosines == (1,0,0,0,0,1):
            return CORONAL
        elif direction_cosines == (0,1,0,0,0,1):
            return SAGITAL
        elif direction_cosines == (1,0,0,0,1,0):
            return AXIAL

        return AXIAL
    
    def __init__(self,path):
        dicom=dcmread(path)
        self.plane = DicomFile.direction_cosines_to_plane(
            dicom[0x0020,0x0037].value
        )
        self.dicom=dicom
        self.path=path
        self.pixels_raw = dicom.pixel_array
        self.pixels=rescale_array(np.float64(self.pixels_raw))
        self.mm_per_pixel=dicom[0x0028,0x0030].value #tuple x,y\
        self.mm_per_slice=dicom[0x0018,0x0050].value
        self.w,self.h=self.pixels.shape
        self.shape=self.pixels.shape
        self.hounsfield=float(dicom[0x0028,0x1053].value)*np.float64(dicom.pixel_array)+float(dicom[0x0028,0x1052].value)

files=natsorted(list(os.listdir(DICOM_DIR)))
dicom_file=DicomFile(f"{DICOM_DIR}\{files[0]}")
w=dicom_file.w
h=dicom_file.h

d=len(files)

cellx,celly=dicom_file.mm_per_pixel[0],dicom_file.mm_per_pixel[1]
cellz=dicom_file.mm_per_slice

axx=cellx*w/2
axy=celly*h/2
axz=cellz*d/2
minax=min(axx,axy,axz)
axx=axx/minax
axy=axy/minax
axz=axz/minax

axs = np.array([axx, axy, axz], dtype=np.float32)
res = np.array([w, h, d],int)

volume=np.zeros((w,h,d),dtype=np.float32)
for z,file in enumerate(files):
    dicom=DicomFile(os.path.join(DICOM_DIR,file))
    volume[:,:,z]=dicom.hounsfield.astype(np.float32).transpose()

volume=rescale_array(volume)

dcos = tuple(map(int,dicom_file.dicom[0x0020,0x0037].value))

cosines = {
    "axial":AXIAL_COSINES,
    "sagital":SAGITAL_COSINES,
    "coronal":CORONAL_COSINES
}[dicom_file.plane]

for i in [0,1,2]:
    if dcos[i] != 0 and cosines[i] != 0 and dcos[i] != cosines[i]:
        volume = np.flip(volume, axis=0)
        break

for i in [3,4,5]:
    if dcos[i] != 0 and cosines[i] != 0 and dcos[i] != cosines[i]:
        volume = np.flip(volume, axis=1)
        break

match dicom_file.plane:

    case "axial":
        volume = volume.transpose((0,2,1))
        axs = axs[[0,2,1]]
        res = res[[0,2,1]]
        pass

    case "sagital":
        volume = volume.transpose((2,1,0))
        axs = axs[[2,1,0]]
        res = res[[2,1,0]]
        pass

    case "coronal":
        volume = volume.transpose((0,1,2))
        axs = axs[[0,1,2]]
        res = res[[0,1,2]]
        pass

volume = np.flip(volume, axis=1)

with open("temp/initial_volume.txt","wb") as fl:

    fl.write((" ".join([str(ax) for ax in axs])).encode('ascii')+b"\n")

    fl.write((" ".join([str(r) for r in res]).encode('ascii')+b"\n"))
    print("Writing data to file...")
    volume = volume.astype(dtype=np.float32)
    for z in range(res[2]):
        for y in range(res[1]):
            for x in range(res[0]):
                fl.write(struct.pack("<f",volume[x,y,z]))
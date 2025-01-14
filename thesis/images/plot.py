import numpy as np
import math
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.pyplot import figure
import glob

figure(figsize=(8, 6), dpi=80)

for filename in glob.glob("*.csv"):
    directivity = pd.read_csv(filename)
    directivity.assign(e=directivity.iloc[0,:])

    phi = np.linspace(0, 2 * np.pi, num=directivity.shape[1], endpoint=True)
    fig, ax = plt.subplots(subplot_kw={'projection': 'polar'})
    plane = directivity.iloc[directivity.shape[0] // 2]
    ax.plot(phi, plane)
    ax.grid(True)
    ax.set_rlabel_position(90)
    plt.savefig(filename.replace(".csv", ".pdf"))


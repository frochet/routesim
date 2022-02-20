"""
20/02/2020
This is the script for calculate expected probability for each node being placed in each layer
"""
import csv
import os
import pandas as pd

def get_layer_prob(path, filename):
    with open(os.path.join(path, filename), 'r') as f:
        csv_reader = csv.reader(f)
        header     = next(csv_reader)
        if header != None:
            positions = [[int(l) for l in row[3:]] for row in csv_reader]

        l_prob = []
        for mix_id, line in enumerate(positions):
            row = [line.count(l) for l in [-1, 0, 1, 2]]
            l_prob.append([e/1000 for e in row])
        
        return l_prob


def matrix_add(lista, listb):
    sum = []
    for (a, b) in list(zip(lista, listb)):
        sum.append([aa+bb for (aa, bb) in list(zip(a, b))])
    return sum


if __name__ == "__main__":

    # INPUT: target layout file
    path = "../results/avg_topos"
    filename = "static_hybrid_layout.csv"

    # get average layer probability
    mix_col_list = ["mix_id", "bandwidth", "malicious"]
    mix_df = pd.read_csv(os.path.join(path, filename), usecols=mix_col_list)
    layer_counts = get_layer_prob(path, filename)
    layer_df = pd.DataFrame(layer_counts, columns=['unselected', 'layer0', 'layer1', 'layer2'])
    df_merged = pd.concat([mix_df, layer_df], axis=1)

    # OUTPUT: write average topo to avg_xxxx.csv under the same directory
    df_merged.to_csv(os.path.join(path, "avg_"+filename), index=True)


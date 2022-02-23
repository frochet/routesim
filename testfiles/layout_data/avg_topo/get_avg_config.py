"""
20/02/2020
This is the script for calculate expected probability for each node being placed in each layer
"""
import csv
import os
import pandas as pd
import glob

def get_layer_prob(path, filename):
    with open(os.path.join(path, filename), 'r') as f:
        csv_reader = csv.reader(f)
        header     = next(csv_reader)
        if header != None:
            positions = [[int(l) for l in row[3:]] for row in csv_reader]

        l_prob = []
        for mix_id, line in enumerate(positions):
            row = [line.count(l) for l in [-1, 0, 1, 2]]
            l_prob.append([e/1000 for e in row]) # probability of being placed in different layers
        
        return l_prob

def avg_layer_counts(path, fnames):
    temp_files = []
    out_path = path+"/output"
    if not os.path.exists(out_path):
        os.makedirs(out_path)

    for filename in fnames:
        # get average layer probability
        mix_col_list = ["mix_id", "bandwidth", "malicious"]
        mix_df = pd.read_csv(os.path.join(path, filename), usecols=mix_col_list)
        layer_counts = get_layer_prob(path, filename)

        layer_df = pd.DataFrame(layer_counts, columns=['unselected', 'layer0', 'layer1', 'layer2'])
        df_merged = pd.concat([mix_df, layer_df], axis=1)

        # OUTPUT: write average topo to avg_xxxx.csv under the same directory
        df_merged.to_csv(os.path.join(path, "temp_"+filename), index=False)
        temp_files.append("temp_"+filename)

    return temp_files

def matrix_add(lista, listb):
    sum = []
    for (a, b) in list(zip(lista, listb)):
        sum.append([aa+bb for (aa, bb) in list(zip(a, b))])
    return sum

def duplicate_split_mixes(path, fnames_list):
    # for each node, split the bw into several pieces according to placing probability
    out_path = path+"/output"
    for fname in fnames_list:
        new_dict = {
            "bandwidth": [],
            "malicious": [],
            "layer": []
        }
        with open(os.path.join(path, fname), 'r') as f:
            csv_reader = csv.DictReader(f)

            for row in csv_reader:
                new_dict["bandwidth"].extend([float(row["bandwidth"])*float(row["layer0"]), 
                                            float(row["bandwidth"])*float(row["layer1"]), 
                                            float(row["bandwidth"])*float(row["layer2"]), 
                                            float(row["bandwidth"])*float(row["unselected"])])
                new_dict["malicious"].extend([row["malicious"]]*4)
                new_dict["layer"].extend([0, 1, 2, -1])

            final_dict = {
                "bandwidth": [],
                "malicious": [],
                "layer":     []
            }
            for i, bw in enumerate(new_dict["bandwidth"]):
                if bw > 0:
                    final_dict["bandwidth"].append(bw)
                    final_dict["malicious"].append(new_dict["malicious"][i])
                    final_dict["layer"].append(new_dict["layer"][i])

            df = pd.DataFrame(final_dict)
            df.insert(0, column="mix_id", value=[i for i in range(len(final_dict["bandwidth"]))])
            df.to_csv(os.path.join(out_path, "{}_topo.csv".format(fname[5:-11])), index=False)


            try:
                fileList = glob.glob(out_path+"/temp*.csv")
                for f in fileList:
                    os.remove(f)
            except:
                pass

def main():
    # INPUT: original layout file
    path = "../results/0218_data_collection/Output/avg_topo"
    filenames = ["smart_rand_bp_30_ffd_layout.csv"]

    duplicate_split_mixes(path, avg_layer_counts(path, filenames))





if __name__ == "__main__":
    main()

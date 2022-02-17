import argparse
import pickle
import os
from multiprocessing import Pool
import time
import csv
from datetime import datetime
from distutils.util import strtobool
import math

parser = argparse.ArgumentParser(description="""Process results from routesim.py.
                                 Output serialized objects for plotting script,
                                 and output the answer to 'How many messages
                                 clients send before getting deanonymed, on
                                 average'""")
parser.add_argument("--in_file", help="The output file produced by\
                    routesim.py", required=True)
parser.add_argument("--outname", help="filename for the pickle storage")
parser.add_argument("--nbr_messages_until_compromise", action="store_true",
                    help="Display the number of messages until compromise, on average")


def parse_log_routesim_simple(filename):
    """
        Parse and process the output produced from the routesim "simple" user model.
        A typical line example:

        1970-01-04 06:05:41 3785 1122,533,234, false

    """
    res = {'nbr_messages_until_compromise': {},
           'time_to_first_compromise': {}}
    with open(filename) as logfile:
        for line in logfile:
            tab = line.split()
            sample_id = int(tab[2])
            is_compromised = strtobool(tab[4])
            # avoid considering a "if" branch when the key exist.
            try:
                res['nbr_messages_until_compromise'][sample_id] += 1
            except KeyError:
                res['nbr_messages_until_compromise'][sample_id] = 0
            if is_compromised and sample_id not in res['time_to_first_compromise']:
                dt = datetime.fromisoformat("{} {}".format(tab[0], tab[1]))
                res['time_to_first_compromise'][sample_id] = dt.timestamp()
    return res

if __name__ == "__main__":

    args = parser.parse_args()
    ## Get all the data file
    print(f'==============Now process the {args.in_file} file================')
    results = parse_log_routesim_simple(args.in_file)
    # add math.inf for uncompromised users:
    for sampleid in results['nbr_messages_until_compromise'].keys():
        if sampleid not in results['time_to_first_compromise']:
            results['time_to_first_compromise'][sampleid] = math.inf
    if args.nbr_messages_until_compromise:
        try:
            #compute the avg for the number of message to send until compromise
            avg_msg = sum(results['nbr_messages_until_compromise'].values())/len(results['nbr_messages_until_compromise'])
            print("How many messages do users send until deanonymized, on average?\
                    {0} messages".format(avg_msg))
            print("{} sample have been compromised".format(len([x for x in results['time_to_first_compromise'].values() if x < math.inf])))
        except ZeroDivisionError:
            print(f"The simulation did not run enough to compromise any user -- something must have been wrong :)")
    with open(args.outname+".pickle", "wb") as outfile:
        # Dump for each user the timestamp of the first compromised path and other processed info
        pickle.dump(results, outfile, pickle.HIGHEST_PROTOCOL)


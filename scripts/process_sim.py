import argparse
import pickle
import os, sys
from multiprocessing import Pool
import time
import csv
from datetime import datetime
from distutils.util import strtobool
import math
import pdb

parser = argparse.ArgumentParser(description="""Process results from routesim.py.
                                 Output serialized objects for plotting script,
                                 and output the answer to 'How many messages
                                 clients send before getting deanonymed, on
                                 average'""")
parser.add_argument("--in_file", help="The output file produced by\
                    routesim.py", required=True)
parser.add_argument("--outname", help="filename for the pickle storage")
parser.add_argument("--format", default="simple", help="tell the parser the expected format") 
parser.add_argument("--nbr_messages_until_compromise", action="store_true",
                    help="Display the number of messages until compromise, on average")
parser.add_argument("--samples", required=True, type=int, help="Number of samples in file")

def parse_log_routesim_async(filename):
    """
        Parse and process the output produced from the "email" user model (and  maybe other in the future)
        A typical line example:

        Date Datetime sampleid requestid message_counter mix1,mix2,mix3,mailbox is compromised
        1970-01-02 11:34:00 0 15363550716705079793 755,640,1007,815 false

        A relationship is considered deanonymized if, for any request between
        user i and user j, at least 1 of the messages ran through a compromised path for each user

    """
    res = {'nbr_messages_until_compromise': {},
           'nbr_emails_until_compromise': {},
           'time_to_first_compromise': {}}
    with open(filename) as logfile:
        tmp = {'message':{}, 'request':{}, 'confirmed': {}}
        counts = {}
        request_compromised = {}
        for line in logfile:
            tab = line.split()
            is_compromised = strtobool(tab[-1])
            sample_id = int(tab[2])
            request_id = int(tab[3]) 
            try:
                counts[sample_id]['count'] += 1
                counts[sample_id]['request'][request_id] = True
            except KeyError:
                counts[sample_id] = {}
                counts[sample_id]['count'] = 0
                counts[sample_id]['request'] = {}
                counts[sample_id]['request'][request_id] = True

            if is_compromised:
                if request_id not in request_compromised:
                    dt = datetime.fromisoformat("{} {}".format(tab[0], tab[1]))
                    timestamp = dt.timestamp()
                    if sample_id not in tmp['message']:
                        tmp['message'][sample_id] = {}
                        tmp['request'][sample_id] = {}
                    request_compromised[request_id] = True
                    tmp['message'][sample_id][request_id] = counts[sample_id]['count']
                    tmp['request'][sample_id][request_id] = len(counts[sample_id]['request'])
                    tmp['confirmed'][request_id] = [sample_id, timestamp]
                    ## don't add multiple times for multiple messages deanonymized in the same request
                if tmp['confirmed'][request_id][0] != sample_id:
                    sample = tmp['confirmed'][request_id][0]
                    timestamp = tmp['confirmed'][request_id][1]
                    if len(tmp['confirmed'][request_id]) == 2:
                        tmp['confirmed'][request_id].append(sample_id)
                    if sample in res['nbr_messages_until_compromise'] and timestamp < res['time_to_first_compromise'][sample]:
                        res['time_to_first_compromise'][sample] = timestamp
                        res['nbr_messages_until_compromise'][sample] = tmp['message'][sample][request_id]
                        res['nbr_emails_until_compromise'][sample] = tmp['request'][sample][request_id]
                    elif sample not in res['nbr_messages_until_compromise']:
                        res['time_to_first_compromise'][sample] = timestamp
                        res['nbr_messages_until_compromise'][sample] = tmp['message'][sample][request_id]
                        res['nbr_emails_until_compromise'][sample] = tmp['request'][sample][request_id]

            
    return res
def parse_log_routesim_sync(filename):
    """
        Parse and process the output produced from the routesim "simple" user model.
        A typical line example:
        
        Date Datetime sample_id mix1,mix2,mix3, is_compromised
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
    if args.format == "sync":
        process_log = parse_log_routesim_sync
    elif args.format == "async":
        process_log = parse_log_routesim_async
    else:
        print(f"Unsupported format: {args.format}")
        sys.exit(-1)
    ## Get all the data file
    print(f'==============Now process the {args.in_file} file================')
    results = process_log(args.in_file)
    # add math.inf for uncompromised users:
    for sampleid in range(0, args.samples):
        if sampleid not in results['time_to_first_compromise']:
            results['time_to_first_compromise'][sampleid] = math.inf
            results['nbr_messages_until_compromise'][sampleid] = math.inf
            if 'nbr_emails_until_compromise' in results:
                results['nbr_emails_until_compromise'][sampleid] = math.inf

    if args.nbr_messages_until_compromise:
        try:
            #compute the avg for the number of message to send until compromise
            avg_msg = sum(results['nbr_messages_until_compromise'].values())/len(results['nbr_messages_until_compromise'])
            print("How many messages do users send until deanonymized, on average?\
                    {} messages for {} compromised users".format(avg_msg, len(results['nbr_messages_until_compromise'])))
            print("{} sample have been compromised".format(len([x for x in results['time_to_first_compromise'].values() if x < math.inf])))
        except ZeroDivisionError:
            print(f"The simulation did not run enough to compromise any user -- something must have been wrong :)")
    with open(args.outname+".pickle", "wb") as outfile:
        # Dump for each user the timestamp of the first compromised path and other processed info
        pickle.dump(results, outfile, pickle.HIGHEST_PROTOCOL)


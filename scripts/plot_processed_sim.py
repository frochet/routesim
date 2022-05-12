"""
Some functions are borrowed from github.com/torps

"""

import os
import pickle
import sys
import numpy
import matplotlib
matplotlib.use('PDF') # alerts matplotlib that display not required
import matplotlib.pyplot
import math
import argparse
import pdb

parser = argparse.ArgumentParser(description="Plot Time-to-first route compromised cdf of the routesim results")

parser.add_argument("--time", action="store_true")
parser.add_argument("--count", action="store_true")
parser.add_argument("--data", nargs="+", help="datapath to the pickle file")
parser.add_argument("--label", nargs="+", help="Line label in the same order than the data")


##### Plotting functions #####
## helper - cumulative fraction for y axis
def cf(d): return numpy.arange(1.0,float(len(d))+1.0)/float(len(d))

## helper - return step-based CDF x and y values
## only show to the 99th percentile by default
def getcdf(data, shownpercentile=0.99):
    data.sort()
    frac = cf(data)
    x, y, lasty = [], [], 0.0
    for i in range(int(round(len(data)*shownpercentile))):
        x.append(data[i])
        y.append(lasty)
        x.append(data[i])
        y.append(frac[i])
        lasty = frac[i]
    return (x, y)

def plot_cdf(lines, line_labels, xlabel, title, location, out_pathname,
    figsize = None, fontsize = 'large'):
    """Saves cdf for given lines in out_name."""
    fig = matplotlib.pyplot.figure(figsize = figsize)
    line_styles = ['-v', '-o', '-s', '-*', '-x', '-D', '-+']
    num_markers = 10
    
    if (line_labels != None):
        i = 0
        for data_points, line_label in zip(lines, line_labels):
            # cut off points with largest value
            data_max = max(data_points)
            data_shown = list(filter(lambda x: x < data_max, data_points))
            shown_percentile = float(len(data_shown)) / len(data_points)
            x, y = getcdf(data_points, shown_percentile)
            matplotlib.pyplot.plot(x, y, line_styles[i % len(line_styles)],
                label = line_label,
                linewidth = 3,
                markevery = int(math.floor(len(x)/num_markers)))
            i += 1
        matplotlib.pyplot.legend(loc=location, fontsize = fontsize)
    else:
        x, y = getcdf(lines)
        matplotlib.pyplot.plot(x, y)
    #matplotlib.pyplot.xlim(xmin=0.0)
    matplotlib.pyplot.ylim(ymin=0.0)
    matplotlib.pyplot.yticks(numpy.arange(0, 1.1, 0.2), fontsize=fontsize)
    matplotlib.pyplot.xticks(fontsize=fontsize)
    matplotlib.pyplot.xlabel(xlabel, fontsize=fontsize)
    matplotlib.pyplot.ylabel('Cumulative probability', fontsize=fontsize)
    matplotlib.pyplot.grid()
    matplotlib.pyplot.tight_layout()

    #matplotlib.pyplot.show()
    matplotlib.pyplot.savefig(out_pathname)

if __name__ == "__main__":

    args = parser.parse_args()
    if not args.time and not args.count:
        print("--count or --time is missing")
        sys.exit(-1)
    data = []
    figsize = (6.4, 3.8)
    fontsize=18
    max_value = 0
    for datapath in args.data:
        with open(datapath, "rb") as file:
            simresults = pickle.load(file)
            this_max_value = max(filter(lambda elem: elem < math.inf, simresults['time_to_first_compromise'].values()))
            if max_value < this_max_value:
                max_value = this_max_value
    #within one week
    if max_value <= 7*24*60*60:
        divider = 60*60
        unit = "[hours]"
    #within one month
    elif max_value <= 30*24*60*60:
        divider = 24*60*60
        unit = "[days]"
    #within one year
    elif max_value <= 12*30*24*60*60:
        divider = 7*24*60*60
        unit = "[weeks]"
    else:
        divider = 30*24*60*60
        unit = "[months]"

    for datapath in args.data:
        with open(datapath, "rb") as file:
            simresults = pickle.load(file)
            if args.time:
                data.append([ float(x)/divider for x in simresults['time_to_first_compromise'].values() ])
            elif args.count:
                if 'nbr_emails_until_compromise' in simresults:
                    data.append([ float(x) for x in simresults['nbr_emails_until_compromise'].values() ])
                else:
                    data.append([ float(x) for x in simresults['nbr_messages_until_compromise'].values() ])
    if args.time:
        plot_cdf(data, args.label, "time to first compromise "+unit, "test", "best", "ttfc_cdf_routesimresults", figsize=figsize, fontsize=fontsize)
    elif args.count and 'nbr_emails_until_compromise' in simresults:
        plot_cdf(data, args.label, "Number of emails sent until compromise", "test", "best", "counts_emails_cdf_routesimsresults", figsize=figsize, fontsize=fontsize)
    else:
        plot_cdf(data, args.label, "Number of messages sent until compromise", "test", "best", "counts_messages_cdf_routesimsresults", figsize=figsize, fontsize=fontsize)



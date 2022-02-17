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

parser = argparse.ArgumentParser(description="Plot Time-to-first route compromised cdf of the routesim results")

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
    figsize = None, fontsize = 'small'):
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
                linewidth = 2,
                markevery = int(math.floor(len(x)/num_markers)))
            i += 1
        matplotlib.pyplot.legend(loc=location, fontsize = fontsize)
    else:
        x, y = getcdf(lines)
        matplotlib.pyplot.plot(x, y)
    matplotlib.pyplot.xlim(xmin=0.0)
    matplotlib.pyplot.ylim(ymin=0.0)
    matplotlib.pyplot.yticks(numpy.arange(0, 1.1, 0.1))
    matplotlib.pyplot.xlabel(xlabel, fontsize=fontsize)
    matplotlib.pyplot.ylabel('Cumulative probability', fontsize=fontsize)
    matplotlib.pyplot.grid()
    matplotlib.pyplot.tight_layout()
    
    #matplotlib.pyplot.show()
    matplotlib.pyplot.savefig(out_pathname)

if __name__ == "__main__":
    
    args = parser.parse_args()
    data = []
    for datapath in args.data:
        with open(datapath, "rb") as file:
            simresults = pickle.load(file)
            data.append([ float(x)/(60*60) for x in simresults['time_to_first_compromise'].values() ])
    plot_cdf(data, args.label, "hours", "test", "best", "cdf_routesimresults")

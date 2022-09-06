# Context

Anonymous communication network designs exist in different flavors to defend
different threat models with various adversarial strengths. Tor is an example of
extremely *performant* design that *probabilistically* resists local
adversaries to some extend, but does not offer any guarantee against a global
passive adversary (GPA). At the other end of this spectrum, academic proposals
such as Atom or XRD offer a provably secure solution against both a GPA and
local attackers, such as insiders (i.e., a fraction of the network controlled
by the adversary).  However, those designs pay a serious performance penalty,
limiting their appeal. In the middle of this situation, we also find designs
such as Loopix or Nym, which are _not_ provably secure against a GPA but
*should* be stronger than Tor. Yet like Tor, they also *probabilistically*
resist local adversaries only to some extend, i.e., the probability to get
compromised is actually far from negligible.

This tool focuses on systems such Loopix or Nym and measures their ability to
resist against the insider threat: an adversary running nodes in the network
and actively trying to deanonymize their users. Eventually, it is possible to
use this tool to test subtle design decisions and have a precise understanding
of how they impact users' anonymity. As a user of a given anonymity network, it
is also possible to use this tool to simulate our own behavior and evaluate its
impact. This could be useful, for example, for a whistleblower to verify and
adapt their behavior to minimize chances to get deanonymized while
communicating critical information.

# Routesim -- Technical goal

This is a behavioural simulator for user activity in a Mixnet. The objective of
this tool is to evaluate the probability of a deanonymization through time,
assuming some level of adversarial activity among the mixes. This probability
of deanonymization is determined by the user behaviour (i.e., how many messages
the user sends, and what sending distribution through time).

This program can take in input behavioural patterns; i.e., a probabilistic
model of the user activity through a period. The simulator can then play this
pattern upon a configured virtual time limit (in days). For example, the
simulator can play a user email pattern within the Mixnet during 100 days.

We read topologies configuration from files, in which each file matchs an epoch
(i.e., an epoch is a timeframe during which the network topology remains the
same). The simulator then output path information for each message sent by each
sampled user (i.e., we apply a Monte Carlo method). As a matter of example, the
"simple" model outputs the following kind of line:

```bash  
1970-01-01 00:44:31 2538 570,260,1007, false  
```   

containing the date, the sample id, the path (mix ids) and whether the route is
fully compromised or not (i.e., whether the user selected PATH_LENGTH malicious mixes).

The timings and number of path sampled depends on the user model selected. E.g.,
for the "simple" user model, one path is sampled every 10 minutes on average
(uniformly random in 5, 15 minutes).

More complex user models and logging mechanisms are also implemented.
Fundamentally, we derive two classes of interactions with the mixnet:
Synchronous one. E.g., sending files to a dropbox, or sending a HTTP request to
a Web Server). And asynchronous ones that mainly capture potential interaction
between anonymous users (e.g., some user send a message to someone's else
mailbox; and the peer can then anonymously fetch the message asynchronously).

# Installing Rust

```bash
curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs | sh
```
See [Other Installation
Methods](https://forge.rust-lang.org/infra/other-installation-methods.html) if
you're on Windows.

Add Cargo to your PATH.

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```
# Compiling and installing routesim

Clone the repository.

```bash
  git clone https://github.com/frochet/routesim.git
```

Compile && install.

```bash
cd routesim
cargo install --path .
```

You should now have routesim installed in ~/.cargo/bin and access to its
interface:

```bash
routesim -h
```
# Examples

## A dummy example

In your terminal, run the following:

```bash
routesim --in-dir testfiles/single_layout --epoch 86401 -u simple --users 2 --days 1 -c 1 -t
```

This instructs routesim to simulate two users during 1 day with the "simple"
user model, and outputs to stdout the route taken during the virtual day.


## A complex usage case: evaluating the impact of your email sending pattern

If you wish to simulate your email-pattern behavior over the Mixnet,
there are a few steps ahead of the simulation itself. Your first need to
extract your data, and post-process them using the available script
located at scripts/process-mailbox.py.

Assuming you have a thunderbird client, install the addon
ImportExportTools NG, and then export your send folder as a .mbox file.
Then, use the script to produce two files containing some processed
information:

```bash
python3 scripts/process-mailbox.py path/to/mbox/file
```

You should then have two .json file in your current directory, named
time_data.json and size_data.json.

## Preparing Bow-Tie topologies

```bash
mkdir -p topologies/bow-tie
cd topologies/bow-tie
../../scripts/split_csv_file.sh ../../testfiles/layout_data/bow_tie/dynamic_hybrid_steady_0.03_layout.csv
```
## Running a Simulation
 
```bash
routesim --timestamps-h time_data.json --sizes-h size_data.json --in-dir topologies/bow-tie --epoch 3600 -u email --users 5000 --days 30 | sed 's/;/\n/g' > output_routesim_data
```

## Parsing & Plotting Results

You'll find scripts to process and plots the simulation results in
the directory scripts/. `process_sim.py` processes the output of the
routesim command and stores relevant summaries serialized in a pickle
file.

```
$ python3 scripts/process_sim.py -h
usage: process_sim.py [-h] --in_file IN_FILE [--outname OUTNAME] [--format FORMAT] [--nbr_messages_until_compromise] --samples SAMPLES

Process results from routesim.py. Output serialized objects for plotting script, and output the answer to 'How many messages clients send before getting deanonymed, on average'

optional arguments:
  -h, --help            show this help message and exit
  --in_file IN_FILE     The output file produced by routesim.py
  --outname OUTNAME     filename for the pickle storage
  --format FORMAT       tell the parser the expected format
  --nbr_messages_until_compromise
                        Display the number of messages until compromise, on average
  --samples SAMPLES     Number of samples in file used in the simulation (--users in routesim)

```

```bash
python3 scripts/process_sim.py --in_file output_routesim_data --outname data_to_plot --format async --samples 5000
```

Now you should have a file named data_to_plot.pickle in your active
directory. You can use `plot_processed_sim.py` to get a visual.

```
$ python3 scripts/plot_processed_sim.py -h
usage: plot_processed_sim.py [-h] [--time] [--count] [--data DATA [DATA ...]] [--label LABEL [LABEL ...]]

Plot Time-to-first route compromised cdf of the routesim results

optional arguments:
  -h, --help            show this help message and exit
  --time
  --count
  --data DATA [DATA ...]
                        datapath to the pickle file
  --label LABEL [LABEL ...]
                        Line label in the same order than the data
```

You can give the script multiple --data and --label for all your
simulations. They will be plotted on the same figure.

```bash
python3 scripts/plot_processed_sim.py --time --data data_to_plot.json.pickle --label simulation_example
```

In this case, only one line is expected.


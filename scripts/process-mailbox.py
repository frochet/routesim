from datetime import datetime
import mailbox
import sys
import json

def process_emails(fname):
    mbox = mailbox.mbox(fname)
    total_messages = 0
    format = "%a, %d %b %Y %H:%M:%S"
    period = 604800
    earliest = datetime.today()
    oldest = datetime(1970,1,1,0,0,0)
    smallest = 604800
    start_of_week_alignment = 60*60*24*4 #Aligning to Monday from Thursday (1/1/1970)
    sending_times = []
    mesg_sizes = []

    for message in mbox:
        sent_time = message['Date']
        if sent_time:
            wkday, day, month, year, timestamp, *_ , = sent_time.split()
            temp_time = wkday + " " + day + " " + month + " " + year + " " + timestamp
            try:
                sent_time = datetime.strptime(temp_time, format)
                if sent_time > oldest:
                    oldest = sent_time
                if sent_time < earliest:
                    earliest = sent_time
                sent_time = (int(sent_time.timestamp()) + start_of_week_alignment) % period   # Number of seconds since 00:00:00 UTC 1/1/1970 modulo period (1 week in seconds)
                sending_times.append(sent_time)
                size = len(message.as_bytes())
                mesg_sizes.append(size)
                total_messages += 1
            except ValueError:
                print("Incorrect date format. Message skipped.")
    num_weeks = (oldest - earliest).days//7
    nbr_sampling = total_messages//num_weeks
    
    print(earliest, oldest, num_weeks, nbr_sampling)

    output_data = {'nbr_sampling': nbr_sampling, 'data': sorted(sending_times)}
    with open('time_data.json', 'w', encoding='utf-8') as f:
        json.dump(output_data, f, ensure_ascii=False, indent=4)

    output_data = {'data': sorted(mesg_sizes), 'nbr_sampling': 0}
    with open('size_data.json', 'w', encoding='utf-8') as f:
        json.dump(output_data, f, ensure_ascii=False, indent=4)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print ("Usage: python3 process-mbox.py mbox_file")
        sys.exit(1)

    process_emails(sys.argv[1])


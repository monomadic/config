#!/bin/bash
#
# Description: Determine speed of write/read on drive command executed on
#

DT=`date +%Y%m%d%H%M%S`
LOG="$HOME/Desktop/Dropbox/Notes/disk_speed_test.log"

for p in {1..10}
do

echo
echo "Run ${p}"
echo "---------------------"
echo "Write Test Running. Please Wait..."
write=$(dd if=/dev/zero bs=2048k of=tstfile count=1024 2>&1 | grep sec | awk '{print $1 / 1024 / 1024 / $5, "MB/sec" }')
echo ${write} >> /tmp/throughput-write
sudo purge
echo "Read Test Running. Please Wait..."
read=$(dd if=tstfile bs=2048k of=/dev/null count=1024 2>&1 | grep sec | awk '{print $1 / 1024 / 1024 / $5, "MB/sec" }')
echo ${read} >> /tmp/throughput-read
sudo purge

echo ""
echo "Write Speed is: $write"
echo "Read Speed is: $read"
echo ""
[ -f tstfile ] && rm tstfile && echo "File tstfile removed"
echo ""

done

AVGWRITE=`cat /tmp/throughput-write | awk '{print $1}' | awk '{s+=$1} END {print s}' | awk '{print $1 / 10 }'`
AVGREAD=`cat /tmp/throughput-read | awk '{print $1}' | awk '{s+=$1} END {print s}' | awk '{print $1 / 10 }'`

echo "################################################"
echo "                                                " | tee -a "$LOG"
echo "Performed on ${PWD} - ${DT}                     " | tee -a "$LOG"
echo "Average Write Speed Over 10 Iterations: ${AVGWRITE} MB/sec" | tee -a "$LOG"
echo "Average Read Speed Over 10 Iterations: ${AVGREAD} MB/sec" | tee -a "$LOG"
echo "                                                " | tee -a "$LOG"
echo "################################################"
echo

[ -f /tmp/throughput-write ] && rm /tmp/throughput-write
[ -f /tmp/throughput-read ] && rm /tmp/throughput-read


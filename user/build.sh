#!/bin/bash -e

IMG=fs.img
MNT=mnt
ROOT=$(git rev-parse --show-toplevel)

PROGS=(cat fib heap init shell sleep)

dd if=/dev/zero of=$IMG bs=1MB count=128
echo -e "n\np\n1\n\n\nt\nc\nw\n" | fdisk $IMG

LO=$(sudo losetup --show -f -P $IMG)
LOP1=${LO}p1

if [ ! -e $LOP1 ]; then
    PARTITIONS=$(lsblk --raw --output "MAJ:MIN" --noheadings ${LO} | tail -n +2)COUNTER=1
    COUNTER=1
    for i in $PARTITIONS; do
        MAJ=$(echo $i | cut -d: -f1)
        MIN=$(echo $i | cut -d: -f2)
        if [ ! -e "${LO}p${COUNTER}" ]; then sudo mknod ${LO}p${COUNTER} b $MAJ $MIN; fi
        COUNTER=$((COUNTER + 1))
    done
fi

sudo mkfs.vfat -F32 $LOP1

mkdir -p $MNT
sudo mount $LOP1 $MNT

trap "sudo umount $MNT; rmdir $MNT; sudo losetup -d $LO" EXIT

for d in ${PROGS[@]}; do
    sudo cp $ROOT/target/aarch64-unknown-none/release/$d.bin $MNT/$d
done

sudo sh -c 'echo "test file for user applications" > $MNT/user.txt'

# TODO: find a general way to fix this
qemu-img resize fs.img 256M
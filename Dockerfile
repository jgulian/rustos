FROM ubuntu:18.04

ADD bin bin

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get -y install git

WORKDIR /build
RUN git clone git://allspark.gtisc.gatech.edu/cs3210-rustos-pub --origin skeleton rustos
WORKDIR /build/rustos

RUN git fetch; git merge skeleton/lab0
RUN bin/setup.sh
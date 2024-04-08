FROM pgrouting/pgrouting:16-3.4-3.6.1

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG C.UTF-8
ENV LC_ALL C.UTF-8
ENV PG_VERSION 16

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
    default-jre \
    sudo \
    osm2pgsql \
    wget \
    unzip \
&& apt-get clean autoclean \
&& apt-get autoremove -y \
&& rm -rf /var/lib/{apt,dpkg,cache,log}/

COPY postgresql.custom.conf . 

COPY run.sh .
CMD ["./run.sh"]

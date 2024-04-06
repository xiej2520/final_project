FROM pgrouting/pgrouting:16-3.4-3.6.1

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG C.UTF-8
ENV LC_ALL C.UTF-8
ENV PG_VERSION 16

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
    sudo \
    wget \
    default-jre \
    unzip \
&& apt-get clean autoclean \
&& apt-get autoremove -y \
&& rm -rf /var/lib/{apt,dpkg,cache,log}/

WORKDIR /tmp/osm2po

COPY osm2po.config .

COPY run.sh .
CMD ["./run.sh"]
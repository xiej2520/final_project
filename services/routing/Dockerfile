FROM pgrouting/pgrouting:16-3.4-3.6.1
ENV DEBIAN_FRONTEND=noninteractive
ENV LANG C.UTF-8
ENV LC_ALL C.UTF-8

RUN apt-get update \
&& apt-get install -y --no-install-recommends \
    osmconvert \
    osm2pgrouting \
&& apt-get clean autoclean \
&& apt-get autoremove -y \
&& rm -rf /var/lib/{apt,dpkg,cache,log}/

RUN adduser --disabled-password --gecos "" router

COPY run.sh /
ENTRYPOINT [ "/run.sh" ]
CMD []
EXPOSE 5432
#!/usr/bin/env bash

set -e

VALHALLA_DATA_DIR=${VALHALLA_DATA_DIR:-./valhalla-data}
FILENAME=${FILENAME:-andorra-latest.osm.pbf}
FILEPATH=$VALHALLA_DATA_DIR/$FILENAME

S3_ENDPOINT="${S3_ENDPOINT:-s3.localhost}"
S3_PORT="${S3_PORT:-3000}"
S3_SCHEME="${S3_SCHEME:-http}"
S3_ACCESS_KEY="${S3_ACCESS_KEY:-root}"
S3_SECRET_KEY="${S3_SECRET_KEY:-password}"
S3_BUCKET="${S3_BUCKET:-valhalla-data}"

DATE=$(date +%Y.%m.%d)
VALHALLA_TILE_ARCHIVE_PREFIX=${VALHALLA_TILE_ARCHIVE_PREFIX:-valhalla-tiles}
VALHALLA_TILE_ARCHIVE=${VALHALLA_TILE_ARCHIVE:-$VALHALLA_TILE_ARCHIVE_PREFIX-$DATE.tar.gz}


function setup() {
  mkdir -p $VALHALLA_DATA_DIR
  mv $FILEPATH $VALHALLA_DATA_DIR/.$FILENAME > /dev/null 2>&1 || true
  rm -rf $VALHALLA_DATA_DIR/*
  mv $VALHALLA_DATA_DIR/.$FILENAME $FILEPATH > /dev/null 2>&1 || true
}

function download_tiles() {
  local now modified

  if [ -f $FILEPATH ]; then
    modified_at=$(stat "$FILEPATH" --format="%Y")
    now=$EPOCHSECONDS

    delta=$((now - modified_at))
    if (( delta < 60 )); then
      return 0;
    fi
  fi

  # wget -O $FILEPATH https://download.geofabrik.de/europe/$FILENAME
  curl -sSfL -o $FILEPATH https://download.geofabrik.de/europe/$FILENAME
}

function build_tiles() {
  docker run -it --rm \
    -u $(id -u):$(id -g) \
    -v $VALHALLA_DATA_DIR:/custom_files \
    -e use_default_speeds_config=True \
    -e force_rebuild=True \
    -e serve_tiles=False \
    -e build_elevation=True \
    ghcr.io/valhalla/valhalla-scripted:latest
}

function edit_config_json() {
  sed -i "s@/custom_files@$VALHALLA_DATA_DIR@g" $VALHALLA_DATA_DIR/valhalla.json
}

function archive_tiles() {
  tar czf $VALHALLA_TILE_ARCHIVE $VALHALLA_DATA_DIR
}

function wait_for_minio() {
  i=0
  until curl -Is --connect-timeout 1 --max-time 2 "$S3_SCHEME://$S3_ENDPOINT:$S3_PORT/minio/health/live" >/dev/null; do
    ((i++))
    if (( i > 10 )); then
      echo "minio is not available! Exiting..."
      exit 1
    fi
    echo "⏳ Waiting for minio to become available..."
    sleep 5
  done
  echo "🌐 Minio is online"
}

function upload_archive() {
  echo "🌱 Starting to seed minio..."
  mc alias set remote "$S3_SCHEME://$S3_ENDPOINT:$S3_PORT" "$S3_ACCESS_KEY" "$S3_SECRET_KEY"
  mc cp -r $VALHALLA_TILE_ARCHIVE remote/$S3_BUCKET
  echo "🟢 Finished seeding minio"
}

function main() {
  setup
  download_tiles
  build_tiles
  edit_config_json
  archive_tiles
  wait_for_minio
  upload_archive
}

main

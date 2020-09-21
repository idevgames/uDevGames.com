#!/bin/sh

set -x xtrace

SERVICE_SU_USER=xpm
SERVICE_HOST=mysteriouspants.com
SERVICE_USER_NAME=udevgames
SERVICE_USER_DIR=/home/udevgames
SERVICE_NAME=udevgames

cargo build --release

if [ $? -neq 0 ]; then
  echo "fix your broken build"
  return -1
fi

# TODO: set the site into maintenance mode, serving a static page

# stop the live service
ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl stop ${SERVICE_NAME}

# backup the live database
scp ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/db/app.sqlite \
    db/backup-$(date --iso-8601=seconds).sqlite
# TODO: backup the live service binary/resources - used in rollback

rsync -avzr target/release/udevgames \
    ${SERVICE_USER_NAME}@${SERVICE_HOST}:${SERVICE_USER_DIR}/udevgames
# TODO: rsync up any ancillary files needed to run (templates, css, etc)

# ensure permissions and file modes
ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo chown \
    ${SERVICE_USER_NAME}:${SERVICE_USER_NAME} \
    ${SERVICE_USER_DIR}
ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} chmod +x ${SERVICE_USER_DIR}/udevgames

# migrate the db
ssh ${SERVICE_USER_NAME}@${SERVICE_HOST} \
    ${SERVICE_USER_DIR}/udevgames \ # TODO: is this the final binary name?
    migrate

if [ $? -neq 0 ]; then
  echo "Migration failed, you should roll back"
  # TODO: automate the rollback
  return -1
fi

ssh ${SERVICE_SU_USER}@${SERVICE_HOST} sudo systemctl start ${SERVICE_NAME}

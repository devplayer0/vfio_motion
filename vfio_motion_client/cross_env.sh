#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null && pwd )"

if [ "$(docker ps --filter 'name=vfio_motion_cross' -qa | wc -l)" == "1" ]; then
	exec docker start -ai vfio_motion_cross
else
	exec docker run -ti --name vfio_motion_cross --mount type=bind,source="$DIR",target=/build/user devplayer0/vfiomotion-build
fi

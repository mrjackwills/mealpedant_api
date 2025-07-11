#!/bin/bash

# run.sh v0.3.0
# 2025-04-17

APP_NAME='mealpedant'

RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

error_close() {
	echo -e "\n${RED}ERROR - EXITED: ${YELLOW}$1${RESET}\n"
	exit 1
}

# $1 any variable name
# $2 variable name
check_variable() {
	if [ -z "$1" ]; then
		error_close "Missing variable $2"
	fi
}

check_variable "$APP_NAME" "\$APP_NAME"

if ! [ -x "$(command -v dialog)" ]; then
	error_close "dialog is not installed"
fi

# $1 string - question to ask
# Ask a yes no question, only accepts `y` or `n` as a valid answer, returns 0 for yes, 1 for no
ask_yn() {
	while true; do
		printf "\n%b%s? [y/N]:%b " "${GREEN}" "$1" "${RESET}"
		read -r answer
		if [[ "$answer" == "y" ]]; then
			return 0
		elif [[ "$answer" == "n" ]]; then
			return 1
		else
			echo -e "${RED}\nPlease enter 'y' or 'n'${RESET}"
		fi
	done
}

set_base_dir() {
	local workspace="/workspaces"
	if [[ -d "$workspace" ]]; then
		BASE_DIR="${workspace}"
	else
		BASE_DIR=$HOME
	fi
}

set_base_dir

# Get the directory of the script
APP_DIR=$(dirname "$(readlink -f "$0")")

DOCKER_DIR="${APP_DIR}/docker"

# Options
PRO=production
DEV=dev

# Containers
SERVER_API="${APP_NAME}_api"
BASE_CONTAINERS=("${APP_NAME}_postgres" "${APP_NAME}_redis")
ALL=("${BASE_CONTAINERS[@]}" "${SERVER_API}")
TO_RUN=("${BASE_CONTAINERS[@]}")

make_db_data() {
	local pg_data="${BASE_DIR}/databases.d/${APP_NAME}/pg_data"
	local redis_data="${BASE_DIR}/databases.d/${APP_NAME}/redis_data"
	for DIRECTORY in $pg_data $redis_data; do
		if [[ ! -d "$DIRECTORY" ]]; then
			echo -e "${GREEN}making directory:${RESET} \"$DIRECTORY\""
			mkdir -p "$DIRECTORY"
		fi
	done
}

make_logs_directories() {
	local logs_dir="${BASE_DIR}/logs.d/${APP_NAME}"
	if [[ ! -d "$logs_dir" ]]; then
		echo -e "${GREEN}making directory:${RESET} \"$DIRECTORY\""
		mkdir -p "$logs_dir"
	fi
}

make_all_directories() {
	make_db_data
	make_logs_directories
}

dev_up() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	echo "starting containers: ${TO_RUN[*]}"
	docker compose -f dev.docker-compose.yml up --force-recreate --build -d "${TO_RUN[@]}"
}

dev_down() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose -f dev.docker-compose.yml down
}

production_up() {
	make_all_directories
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose up -d

}

production_down() {
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose -f docker-compose.yml down
}

production_rebuild() {
	make_all_directories
	cd "${DOCKER_DIR}" || error_close "${DOCKER_DIR} doesn't exist"
	docker compose up -d --build
}

select_containers() {
	cmd=(dialog --separate-output --backtitle "Dev containers selection" --keep-tite --checklist "select: postgres + redis +" 14 80 16)
	options=(
		1 "${SERVER_API}" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices; do
		case $choice in
		0)
			exit
			;;
		1)
			TO_RUN=("${TO_RUN[@]}" "${SERVER_API}")
			;;
		esac
	done
	dev_up
}

# Checkout the latest main code, and then make new branch based on latest semver tag
git_pull_branch() {
	git checkout -- .
	git checkout main
	git pull origin main
	git fetch --tags
	latest_tag=$(git tag | sort -V | tail -n 1)
	git checkout -b "$latest_tag"
	sleep 5
}

pull_branch() {
	GIT_CLEAN=$(git status --porcelain)
	if [ -n "$GIT_CLEAN" ]; then
		echo -e "\n${RED}GIT NOT CLEAN${RESET}\n"
		printf "%s\n" "${GIT_CLEAN}"
	fi
	if [[ -n "$GIT_CLEAN" ]]; then
		if ! ask_yn "Happy to clear git state"; then
			exit
		fi
	fi
	git_pull_branch
	main
}

run_migrations() {
	if ask_yn "run init_postgres.sh"; then
		docker exec -it "${APP_NAME}_postgres" /docker-entrypoint-initdb.d/init_postgres.sh "migrations"
	fi
}

main() {
	cmd=(dialog --backtitle "Start ${APP_NAME} containers" --keep-tite --radiolist "choose environment" 14 80 16)
	options=(
		1 "${DEV} up" off
		2 "${DEV} down" off
		3 "${PRO} up" off
		4 "${PRO} down" off
		5 "${PRO} rebuild" off
		6 "pull & branch" off
	)
	choices=$("${cmd[@]}" "${options[@]}" 2>&1 >/dev/tty)
	exitStatus=$?
	clear
	if [ $exitStatus -ne 0 ]; then
		exit
	fi
	for choice in $choices; do
		case $choice in
		0)
			exit
			;;
		1)
			select_containers
			run_migrations
			break
			;;
		2)
			dev_down
			break
			;;
		3)
			echo "production up: ${ALL[*]}"
			production_up
			run_migrations
			break
			;;
		4)
			production_down
			break
			;;
		5)
			production_rebuild
			run_migrations
			break
			;;
		6)
			pull_branch
			break
			;;
		esac
	done
}

main

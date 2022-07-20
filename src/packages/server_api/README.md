
<p align="center">
	<h1 align="center">mealpedant server: api</h1>
</p>

<p align="center">
	The backend that powers <a href='https://www.mealpedant.com' target='_blank' rel='noopener noreferrer'>mealpedant.com</a>, via <a href='https://api.mealpedant.com' target='_blank' rel='noopener noreferrer'>api.mealpedant.com</a>
</p>
<p align="center">
	Built in <a href='https://www.typescriptlang.org/' target='_blank' rel='noopener noreferrer'>Typescript</a>,
	for <a href='https://nodejs.org/en/' target='_blank' rel='noopener noreferrer'>Node.js</a>,
	with <a href='https://www.postgresql.org/' target='_blank' rel='noopener noreferrer'>PostgreSQL</a>,
	<a href='https://redis.io/' target='_blank' rel='noopener noreferrer'>Redis</a>,
	<a href='https://rabbitmq.com/' target='_blank' rel='noopener noreferrer'>RabbitMQ</a>, and 
	and <a href='https://www.nginx.com/' target='_blank' rel='noopener noreferrer'>NGINX</a>
</p>

## Required software

1) <a href='https://www.postgresql.org/' target='_blank' rel='noopener noreferrer'>PostgreSQL</a> - database storage
2) <a href='https://redis.io/' target='_blank' rel='noopener noreferrer'>Redis</a> - cache layer
3) <a href='https://www.rabbitmq.com/' target='_blank' rel='noopener noreferrer'>RabbitMQ</a> - message broker
4) <a href='https://gnupg.org/' target='_blank' rel='noopener noreferrer'>gpg</a> - enable encryptions of database backups
5) <a href='https://www.letsencrypt.org/' target='_blank' rel='noopener noreferrer'>Let's Encrypt</a> & <a href ='https://certbot.eff.org/' target='_blank' rel='noopener noreferrer'>certbot</a> (or similar) - all network data over https
6) <a href='https://nodejs.org/en/' target='_blank' rel='noopener noreferrer'>Node.js</a> - runtime
7) <a href='https://www.nginx.com/' target='_blank' rel='noopener noreferrer'>NGINX</a> - reverse proxy, ideally using brotli and gzip compression

<br><br>
Suggested locations for directories required by Meal Pedant

| directory | reason|
| --- | --- |
|```/nodeuser/mealpedant/```					| Location of the node app|
|```/srv/backup/mealpedant/```					| Location of backups ```chmod 775; chown user:srv```
|```/srv/log/mealpedant/```						| Location of logs ```chmod 775; chown user:srv```|
|```/srv/www/static_mealpedant/original/```		| Location for original photo uploads|
|```/srv/www/static_mealpedant/converted/```	| Location for converted photos uploaded|
|```/var/www/letsencrypt/```					| Acme challange storage location, used by certbot for TLS certificates|

<br><br>
File that are required by Meal Pedant

| file | reason|
|---|---|
|```./.env```								| enviromental variables, make sure in production mode|
|```~./.pgpass``` 							| In format ```[host]:[port]:[postgres_db]:[postgres_user]:[postgres_password]``` chmod 0600, required by backup scripts|
|```~/.mealpedant_gpg_passfile```			| Used to encrypt backup files |
<br>

## Build step

1) ```groupadd srv``` - add srv group
2) ```adduser nodeuser``` - add user nodeuser
3) ```sudo passwd -a nodeuser srv``` - add nodeuser to "srv" group
4) Make sure have correct ownership and permissions
    1) ```sudo chown -R [some_sudo_user]:srv /srv/backup/mealpedant``` 
    2) ```sudo chown -R [some_sudo_user]:srv /srv/log/mealpedant```
    3) ```sudo chmod 770 /srv/backup/mealpedant```
    4) ```sudo chmod 770 /srv/log/mealpedant```
5) Create database with ./sql/db_init.sql
6) ```bash build.sh``` - when on main branch compile typescript and install all node modules
7) ```npm run domains``` - insert all the banned email domains into the database
8) ```sudo ./systemd/install.sh``` Install the backup services
<br>

## Run step
a) ```pm2 start pm2.config.js``` load up into pm2

*or*

b) ```node dist``` run in shell directly

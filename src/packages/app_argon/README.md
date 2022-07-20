
<p align="center">
	<h1 align="center">mealpedant - app:argon</h1>
</p>

<p align="center">
	Argon password hasher & validator internal app for <a href='https://www.mealpedant.com' target='_blank' rel='noopener noreferrer'>mealpedant.com</a>
</p>
<p align="center">
	Built in <a href='https://www.typescriptlang.org/' target='_blank' rel='noopener noreferrer'>Typescript</a>, for <a href='https://nodejs.org/en/' target='_blank' rel='noopener noreferrer'>Node.js</a>, with <a href='https://www.rabbitmq.com/' target='_blank' rel='noopener noreferrer'>RabbitMQ</a>
</p>

## Required software

1) <a href='https://www.rabbitmq.com/' target='_blank' rel='noopener noreferrer'>RabbitMQ</a> - messaging service
2) <a href='https://nodejs.org/en/' target='_blank' rel='noopener noreferrer'>Node.js</a> - runtime


| directory | reason|
| --- | --- |
|```~/app_argon```						| Location of the node app|
|```/var/log/mealpedant/app_argon```	| Location of logs|



File that are required by mealpedant
| file | reason|
|---|---|
|```./.env```		| enviromental variables, make sure in production mode|

## Build step
1) ```bash build.sh``` - when on main branch compile typescript and install all node modules using build process

## Run step
a) ```pm2 start pm2.config.js``` load up into pm2

*or*

b) ```node dist/index``` run in shell directly
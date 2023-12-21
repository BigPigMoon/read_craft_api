# ReadCraft API server

## Start postgres db

```
$ docker run -e POSTGRES_PASSWORD=root -e POSTGRES_USER=bpm -e POSTGRES_DB=rc_api -p 5432:5432 postgres:1
```

## postgres db connection string

```
DATABASE_URL="postgres://bpm:root@localhost:5432/rc_api"
```


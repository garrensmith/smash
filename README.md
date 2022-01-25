# Smash

A prisma query-engine concurrency runner. Smash can be used to run concurrent requests against the prisma query engine.
Currently it has workloads designed to test the interactive transactions in the query engine.

## Options

```
Smash - A query engine concurrent simulator

USAGE:
    smash [OPTIONS]

OPTIONS:
    -c, --concurrency <CONCURRENCY>    [default: 10]
    -h, --help                         Print help information
    -i, --iterations <ITERATIONS>      Number of times to run it [default: 1]
    -n, --name <NAME>                  [default: simple]
    -t, --timeout <TIMEOUT>            Timeout in milliseconds for a transaction [default: 1000]
    -V, --version                      Print version information
    -w, --wait <WAIT>                  Wait time for a connection in milliseconds [default: 5000]
```

A basic run would be:

```
smash -c 10 -n simple -i 3 -t 1000 -w 5000
```

That would run the simple workload with 10 concurrent connections and it will do that 3 times. A transaction will timeout after 1s and will wait 5s to get a database connection.

## Workloads

At the moment there are two workloads:

- `simple` - A simple decrement update and batch update that will commit
- `mixed` - A workload that will rollback half of the operations and some just leave to timeout

## Usage

Copy the below schema into `dev_datamodel.prisma` in the `prisma-engines` repo.

```
datasource db {
  provider = "postgres"
  url      = "postgresql://postgres:prisma@localhost:5434"
}

generator js {
  provider        = "prisma-client-js"
  previewFeatures = ["fullTextSearch", "fullTextIndex", "interactiveTransactions"]
}

model Account {
  id      Int    @id @default(autoincrement())
  email   String @unique
  balance Int
}
```

run:

```
$ make push-schema && make qe
```

Then run Smash.

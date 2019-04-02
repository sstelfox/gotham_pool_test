# Gotham Database Pooling Test

This was a quick and dirty test to see if I could get connection pooling
working with the gotham web framework and compare whether it was worth it
compared to setting up the connection during each handler initialization.

The three endpoints are of this simple webservice are:

* http://127.0.0.1:9292/ - `fixed_handler`, the control does not make any
  database connection does all of the work that isn't related to interactions
  with Redis.
* http://127.0.0.1:9292/direct - `direct_handler`, establishes a new connection
  to redis with every request increments a counter and returns it as part of
  the request.
* http://127.0.0.1:9292/pool - `pool_handler`, use an `r2d2` pool that has been
  pre-setup and checks out a connection with each request. Still increments a
  single key and formats and returns the result.

The pool is a bit naive and likely could be improved by writing custom
middleware that clones the pool for each request ahead of time rather than
using a bulky mutex around it and sharing the pool object itself each request.
This was just the fastest / most naive way to perform a basic test.

This should work with any database that can be pooled with `r2d2` such as
PostgreSQL by replacing the appropriate setup.

## Test Results

The following was the results of the test using `ab -t 30 ${ENDPOINT}` on my
personal laptop compiled in release mode with `rustc 1.33.0-nightly (7164a9f15
2019-01-21)`. Your results may vary from mine but these results will likely
give you a rough idea of the differences.

|          | Fixed     | Direct   | Pool     |
| -------: | :-------: | :------: | :------: |
| Reqs/sec | 10,428.94 | 1,513.19 | 5,204.85 |

Even with this naive pool implementation it clearly is a major improvement.

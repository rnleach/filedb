# filedb

A layer on top of SQLITE3 that internally stores files based on a key and time stamp.

 - The key original use case intended the key to be a file name, not including directories, but 
really any text could be used. The point is keys do not need to be unique.
 - The time stamp is NOT provided by this crate, the user must provide it. 
 - The time stamp and key together must be unique, or in database language they are the primary key.
 - The [chrono](https://crates.io/crates/chrono) crate is used for time stamps.


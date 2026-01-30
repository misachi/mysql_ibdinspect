`mysql_ibdinspect` is a `Rust` tool for parsing MYSQL space files(mostly .ibd formats). It parses the pages and prints the contents. I am using this as a learning tool to expand my knowledge of MYSQL internals. A more mature and stable tool was made by [Jeremy Cole](https://github.com/jeremycole/innodb_ruby) a couple of years ago, in `Ruby`.


## Usage
If you have `Rust` installed on your system, clone the repository and run the command as:
```
cargo run -- -f <path_to_ibd_file>  # Replace path_to_ibd_file with the correct .ibd file path
```

Sample results for the first page in a space file:
```
Reading Filespace Header:
Filespace ID: 14
Page Number: 0
Server Version: 90500
Space ID: 14
Space Size (in pages): 15872
Page Type: FIL_PAGE_TYPE_FSP_HDR
Previous Page Number: FIL_NULL (This is the first page)
Next Page Number: 1

Index Page Header Fields:
Number of slots in page directory: 0
Pointer to record heap top: 14
Number of records in the heap: 0
Pointer to start of page free record list: 0
Number of bytes in deleted records: 0
Pointer to the last inserted record: 15872
Last insert direction: 0
Number of consecutive inserts to the same direction: 15488
Number of user records on the page: 0
Highest id of a trx which may have modified a record on the page: 16417
Level of the node in an index tree: 3
Index id where the page belongs: 0
```
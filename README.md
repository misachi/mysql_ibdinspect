`mysql_ibdinspect` is a `Rust` tool for parsing MYSQL space files(mostly .ibd formats). It parses the pages and prints the contents. I am using this as a learning tool to expand my knowledge of MYSQL internals. A more mature and stable tool was made by [Jeremy Cole](https://github.com/jeremycole/innodb_ruby) a couple of years ago, in `Ruby`.


## Usage
If you have `Rust` installed on your system, clone the repository and run the command as:
```
cargo run -- -f <path_to_ibd_file> -n <page_number_to_read> -r <number_of_records_to_display>  # where path_to_ibd_file is the .ibd file path, page_number_to_read is the page index to read(e.g from 0 - Space Size (in pages)), number_of_records_to_display is the number of records in the page to print to screen
```

Sample results for the first page using the command `cargo run --bin mysql_ibdinspect -- -f /home/user/mysql_ibdinspect/sbtest1.ibd -n 0 -r 10` (reads the first page and displays 10 records, if found):
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

Sample results for an index page to display first five records `cargo run --bin mysql_ibdinspect -- -f /home/user/mysql_ibdinspect/sbtest1.ibd -n 7 -r 5`
```
Reading Filespace Header:
Filespace ID: 14
Page Number: 8
Page Type: FIL_PAGE_INDEX
Previous Page Number: 7
Next Page Number: 9

Index Page Header Fields:
Number of slots in page directory: 19
Pointer to record heap top: 15304
Number of records in the heap: 75
Pointer to start of page free record list: 0
Number of bytes in deleted records: 0
Pointer to the last inserted record: 15103
Last insert direction: 2
Number of consecutive inserts to the same direction: 72
Number of user records on the page: 73
Highest id of a trx which may have modified a record on the page: 0
Level of the node in an index tree: 0
Index id where the page belongs: 0
Page is compact: true
Reached leaf level: 0
Data: {
  "c": "71324633764-28566246885-71866095928-68687703310-21646596258-33951331865-05657942146-49812958383-15899513446-92071347977 ",
  "id": 183,
  "k": 247675,
  "pad": "08124836352-39130955895-90473680355-13103594089-21335738640 "
}
Data: {
  "c": "30937840880-30528153806-46333735476-52205021482-12049401561-78931784419-88171434500-75065388775-44172767233-85113010903 ",
  "id": 184,
  "k": 943266,
  "pad": "53224534151-21601268026-26672810425-73109338712-27700103828 "
}
Data: {
  "c": "64566777852-45996873816-09409516716-59318105548-04262603295-82720365064-77030587683-86279949085-33494479141-93788195970 ",
  "id": 185,
  "k": 120624,
  "pad": "04279352585-64418513035-09881122874-31767976120-44516193747 "
}
Data: {
  "c": "79804112894-86067394141-06054308432-46482608015-31065217893-85057279766-90624229528-69729489732-17254557856-95081855560 ",
  "id": 186,
  "k": 657231,
  "pad": "02717602501-24767961292-25554319576-06729775679-92821505050 "
}
Data: {
  "c": "66831863137-56653998416-81918750907-51137340964-72369316887-22942925279-46402869097-62788789012-26340619554-64538782149 ",
  "id": 187,
  "k": 419328,
  "pad": "88937855937-39768457801-30941651306-08459497100-30135251983 "
}
```
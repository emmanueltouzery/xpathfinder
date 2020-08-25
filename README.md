# xpathfinder

At work we're using the [xmlunit](https://www.xmlunit.org/) tool to compare XML
files in tests.

It outputs things like:

```
Expected text value 'true' but was 'false' - comparing <hasCash ...>true</hasCash> at /fareTablesDto[1]/fareTables[1]/fareTable[1]/riderTypes[1]/riderType[1]/hasCash[1]/text()[1] to <hasCash ...>false</hasCash> at /fareTablesDto[1]/fareTables[1]/fareTable[1]/riderTypes[1]/riderType[1]/hasCash[1]/text()[1] (DIFFERENT)

```

This is very useful, however it's very annoying to navigate large XML files
searching for the xpaths it's pointing to.

With this tool:

```
$ xpathpathfinder file.xml "/fareTablesDto[1]/fareTables[1]/fareTable[1]/riderTypes[1]/riderType[1]/hasCash[1]"
Found xpath at position: 969:11
```

You get line and column number in the file for that XPATH.

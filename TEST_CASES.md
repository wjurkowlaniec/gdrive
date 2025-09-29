# Gdrive  test cases 

`/tmp/gdrive_test` is base directory for tests.


Structure inside 

```
f0.zip
/1
 - f1.zip
 - f2.zip
/2
 (empty)

```

Do all tests in `~/tmp/gdrive-tests`. Clear after each use, and fill it with echo "piwo" > f9.zip (just dummy content for testing). 
DO all remote tests on /tmp/gdrive_test
# Functions:

##  display tree

`gdrive tree /tmp`

should give the tree formatted like tree command, but also with date modified (short date) and size 

## Download files

files or directories or just recursively,  with -r flag. If not ,it doesn't do recursive, so if you download just files without directories, by default doing /path/ is something like /path/* in bash, so it will cp but not subdirs

`gdrive pull /tmp/gdrive_test ~/tmp/gdrive-tests` should download just f0.zip  to ~/tmp/gdrive-tests dir  
`gdrive pull /tmp/gdrive_test` should download just f0.zip  to current directory 
`gdrive pull -f /tmp/gdrive_test/` should download all files 

# create directory 

`gdrive mkdir /tmp/gdrive_test/3` should create dir 3  inside base dir 

# Upload files 

`gdrive push f9.zip /tmp/gdrive_test/2` should copy file to dir 2 


`gdrive push . /tmp/gdrive_test/2` should copy all files from current dir to dir 2 without recursive 

`gdrive push -r . /tmp/gdrive_test/2` should copy all files from current dir to dir 2 recursively 

`gdrive push /some/dir/*.jpg  /tmp/gdrive_test/2` should copy all jpeg files to dir 2 

# Deleting 

`gdrive rm /some/dir/file` should remove files

# Bash completion

There should be something like bash completion for remote dirs, like scp has. 

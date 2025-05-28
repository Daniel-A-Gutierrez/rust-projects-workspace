# Folder Cleaner
Ok i know this could be done easily enough in bash but I felt like making a cli tool i guess.

Matches files and folders in a directory by size, filename, and extension, and either moves them or deletes them.

Its normal to see several "x no longer exists" when deleting folders recursively - the file was initially identified but its parent
folder matched the specified conditions too and so it was deleted early. 
# dp - duplicate files

Creates duplicate(s) of file(s) based on file name(s).

## Usage 

```
dp meeting-11-07.org
```

Copies above file into a file with the name `meeting-11-12.org`, given today is November, 12th.

## Todo

- [x] Increment based duplication rules
- [ ] Improved UX
  - [ ] Help to print duplication rules
  - [x] Early exit options in multi-file duplications
- [ ] Robust error handling
- [ ] Tighter date rules which can catch invalid date, month, etc. values
- [ ] Configurable rules(?)

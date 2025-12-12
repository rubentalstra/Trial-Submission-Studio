%* CSV2SAS version 1.2 ;
%let __CSV2SAS_VERSION=1.2;

%macro get_filenames(path, prefix, out=__CSV2SAS_files);
  %* Use a pipe access device to get a list of all CSV files that begins with the specified prefix ;
  %if %length(&prefix)%then filename magritte pipe "dir ""&path.\&prefix.*.csv"" /b";
  %else %if %length(&infile) %then filename magritte pipe "dir ""&path.\&infile"" /b";
  ;
  
  %* Store the names of all CSV files in a dataset ;
  data &out.;
    length filename $256;
    infile magritte length=reclen;
    input filename $varying256. reclen;
  run;

  %* Exclude META and CODELIST files ;
  data &out.;
    set &out.;

    if filename eq "&prefix.Items.csv" then delete; 
    if filename eq "&prefix.CodeLists.csv" then delete;
  run;

  %* Deassign the filename for the pipe access device ;
  filename magritte clear;
%mend get_filenames;

%macro ReadMetaDataFile(file,ds);

  filename FILE_IN "&file.";
  filename FILE_TMP temp;

  data _null_;
    infile FILE_IN delimiter='09'x lrecl=32767 missover dsd firstobs=2; ** Discard the label row (first row);
    file FILE_TMP lrecl=32767;
    attrib line length=$32767;
    input line $;
    line = transtrn(line,'""',trimn(''));
    put line;
  run;

  proc import datafile=FILE_TMP out=&ds. dbms=csv replace;
    getnames=yes;
    guessingrows=10000;
    datarow=2;
  run;
  
  filename FILE_IN clear;
  filename FILE_TMP clear;

%mend ReadMetaDataFile;

%macro CreateSasFormats(file);

  %ReadMetaDataFile(&file.,CSV_FMTS);

  data CSV_FMTS (drop=DataType);
    set CSV_FMTS(rename=(FormatName=FMTNAME CodeValue=Start CodeText=Label));
	HLO = ' ';
    if DataType='integer' then TYPE='N';
    else if (DataType='text' or DataType='string') then TYPE='C';
  run;

  proc format cntlin=CSV_FMTS; run;

  proc delete data=CSV_FMTS; run; quit;

%mend CreateSasFormats;

%macro CreateMetaData(filename,formname);

  %** Create a temporary file that is a copy of the original file with all "" replaced with nothing ;
  %** Apparently proc import does not like empty strings ;
  filename exTemp1 temp; 

  data _null_;
    infile &filename. delimiter='09'x lrecl=32767 missover dsd firstobs=1;
    file exTemp1 lrecl=32767;
    attrib line length=$32767;
    input line $;
    line = transtrn(line,'""',trimn(''));
    put line;
  run;

  %let filename =exTemp1;

  %* Read data from CSV file, using tab as delimiter to get one entry per row ;
  data __CSV2SAS_chk;
    infile &filename. delimiter='09'x lrecl=32767 missover dsd firstobs=1;
    attrib  laban length=$32767.;
    input laban $;
  run;

  %* Number of data rows in dataset ;
  proc sql noprint;
    select nobs into :nrows from dictionary.tables where libname eq 'WORK' and memname eq '__CSV2SAS_CHK';
  quit;
  
  %* Import data from CSV file to get the variable labels and names only;
  proc import datafile=&filename. out=__CSV2SAS_header dbms=csv replace;
    getnames=no;
    guessingrows=2;
    datarow=1;
  run;

  %* Create a dataset with metadata for the header row ;
  %* This dataset will only be used in case there is no data to read in ;
  proc sql;
    create table __CSV2SAS_cmeta (keep=name type format informat)
      as select * from dictionary.columns where libname eq 'WORK' and memname eq '__CSV2SAS_HEADER';
  quit;

  %* Transpose (the first two rows of the data including labels/names) to obtain one row per variable, 
  %* with the desired variable name and label in separate columns ;
  proc transpose data=__CSV2SAS_header (obs=2) out=__CSV2SAS_header2 name=OLDNAME;
      var VAR:;
  run;

  data __CSV2SAS_header3 (drop=COL1 COL2);
    format OLDNAME $8. NEWNAME $32. NEWLABEL $255.;
    set __CSV2SAS_header2;
    NEWLABEL=input(COL1,$255.);
    NEWNAME=input(COL2,$32.);
  run;

  %if &nrows. gt 2 %then %do;

  %* Import data from CSV file (not including the rows containing the variable labels and names) ;
  proc import datafile=&filename. out=__CSV2SAS_data0 dbms=csv replace;
    getnames=no;
    guessingrows=%eval(&nrows.+1);
    datarow=3;
  run;

  %AssertEqualNumberOfVariables(__CSV2SAS_DATA0,__CSV2SAS_HEADER);

  proc sql;
    create table __CSV2SAS_cmeta (keep=name type format informat)
      as select * from dictionary.columns where libname eq 'WORK' and memname eq '__CSV2SAS_DATA0';
  quit;

  proc delete data=__CSV2SAS_data0; run; quit;

  %end;

  data __CSV2SAS_meta;
    order+1;
    merge __CSV2SAS_cmeta __CSV2SAS_header3;
  run;

  %*** ;
  data __&formname._meta;
    set __CSV2SAS_Items ;
  run;

  proc sort data=__CSV2SAS_meta(rename=(NEWNAME=ID)); by ID; run;
  proc sort data=__&formname._meta; by ID; run;

  data __CSV2SAS_meta;
    merge __CSV2SAS_meta (in=ok) __&formname._meta;
    by ID;
    if ok;
  run;

  proc sort data=__CSV2SAS_meta(rename=(ID=NEWNAME)); by order; run;

  %* Delete temporary datasets ;
  proc delete data=__CSV2SAS_chk __CSV2SAS_header __CSV2SAS_header2 __CSV2SAS_header3 __CSV2SAS_cmeta __&formname._meta; run; quit;

  filename exTemp1 clear;

%mend CreateMetaData;

%macro AssertEqualNumberOfVariables(ds1,ds2);
  %* Number of variables in each of the output datasets from the two proc imports (with/without variable names and labels) ;;
  proc sql noprint;
      select nvar into :csv2sas_nvar1-:csv2sas_nvar2 from dictionary.tables where libname eq 'WORK' and memname in ("&ds1.","&ds2.");
  quit;

  %let diff = %eval(&csv2sas_nvar1 - &csv2sas_nvar2);

  %* Compare the number of variables in write comments to the log in case of discrepancies ;;
  %if &diff ne 0 %then %do;
    %put WARNING: &file was not imported successfully.;
    %put COMMENT: There is probably a label that contains a comma.;
    %abort cancel;
  %end;

%mend AssertEqualNumberOfVariables;

%macro runquit;
; run; quit;
%if &syserr. ne 0 %then %do;
  %abort cancel;
%end;
%mend runquit;

%macro CreateCommandsDataSet(ds);

  %* Get variable attributes from the imported data and define commands for the upcoming call execute session ;;
  proc sql noprint;
    create table __CSV2SAS_commands as
      select "data &ds.;" as command from __CSV2SAS_dummy

      outer union corr
        select "infile CSV_IN delimiter='2C'x lrecl=32767 missover dsd firstobs=3; " as command from __CSV2SAS_dummy

      outer union corr
        select 'attrib ' as command from __CSV2SAS_dummy

      outer union corr
        select  case
                  when DataType eq 'integer' and FormatName ne '' then catx(' ',NEWNAME,cats('informat=','8.'),cats('format=',FormatName,'.'))
                  when (DataType eq 'text' or DataType eq 'string') and FormatName ne '' and ContentLength eq '0' then catx(' ',NEWNAME,cats('informat=','$',1,'.'),cats('format=',FormatName,'.'))
                  when (DataType eq 'text' or DataType eq 'string') and FormatName ne '' and not missing(ContentLength) then catx(' ',NEWNAME,cats('informat=','$',ContentLength,'.'),cats('format=',FormatName,'.'))
                  when DataType eq 'integer' or DataType eq 'double' then catx(' ',NEWNAME,cats('informat=','best32.'),cats('format=','best32.'))
                  when (DataType eq 'text' or DataType eq 'base64Binary') and ContentLength eq '0' then catx(' ',NEWNAME,cats('informat=','$',1,'.'),cats('format=','$',1,'.'),cats('length=$',1))
                  when (DataType eq 'text' or DataType eq 'base64Binary') and not missing(ContentLength) then catx(' ',NEWNAME,cats('informat=','$',ContentLength,'.'),cats('format=','$',ContentLength,'.'),cats('length=$',ContentLength))
                  when DataType eq 'datetime' then catx(' ',NEWNAME,cats('informat=','$',16,'.'),cats('format=','$',16,'.'),cats('length=$',16))
                  when DataType eq 'date' then catx(' ',NEWNAME,cats('informat=','$',10,'.'),cats('format=','$',10,'.'),cats('length=$',10))
                  when DataType eq 'time' then catx(' ',NEWNAME,cats('informat=','$',5,'.'),cats('format=','$',5,'.'),cats('length=$',5))
                  when type eq 'num'  then catx(' ',NEWNAME,cats('informat=',informat),cats('format=',format))
                  when type eq 'char' then catx(' ',NEWNAME,cats('informat=','$',input(scan(informat,1,'$.'),8.),'.'),cats('format=','$',input(scan(format,1,'$.'),8.),'.'))
                end 
          as command from __CSV2SAS_meta

      outer union corr
        select ';' as command from __CSV2SAS_dummy

      outer union corr
        select 'input' as command from __CSV2SAS_dummy

      outer union corr
        select  case
                  when (DataType eq 'integer' or DataType eq 'double') and type eq 'char' then NEWNAME /* SAS defaults to char when variable contains no data */
                  when type eq 'num'  then NEWNAME
                  when type eq 'char' then catx(' ',NEWNAME,'$')
              end
          as command from __CSV2SAS_meta

      outer union corr
        select ';' as command from __CSV2SAS_dummy

      outer union corr
        select case
              when FormatName ne '' then catx(' ', 'drop', substr(NEWNAME, 1, length(NEWNAME)-2),';')
              else ''
              end
        as command from __CSV2SAS_meta

      outer union corr
        select case
              when FormatName ne '' then catx(' ', 'rename', cats(NEWNAME,'=',substr(NEWNAME, 1, length(NEWNAME)-2)), ';')
              else ''
              end
        as command from __CSV2SAS_meta

    outer union corr
       select 'attrib ' as command from __CSV2SAS_dummy

      outer union corr
        select case 
          when FormatName ne '' then NEWNAME || " label='" || %trim(%quote(tranwrd(tranwrd(substr(NEWLABEL, 1, length(NEWLABEL)-7),'"','""'), "'", "''"))) || "'"
          else NEWNAME || " label='" || %trim(%quote(tranwrd(tranwrd(NEWLABEL, '"', '""'), "'", "''"))) || "'"
        end
        as command from __CSV2SAS_meta

      outer union corr
        select ';' as command from __CSV2SAS_dummy

      outer union corr
        select 'run;' as command from __CSV2SAS_dummy;
  quit;

%mend CreateCommandsDataSet;

%macro get_dataset_name(file, prefix, lib);
  %let ds = %trim(%scan(%sysfunc(transtrn(&file.,&prefix.,%str())),-2,%str(.)));
  &lib..&ds.; 
%mend get_dataset_name;

%macro doWork(path,prefix);
  %* Get all .csv files in folder ;
  %get_filenames(&path., &prefix.);

  proc sql noprint;
    select filename into :filelist separated by '¤' from __CSV2SAS_files;
   quit;
  
  %* Create a dummy dataset ;;
  proc sql noprint;
    create table __CSV2SAS_dummy (_dummy_ char 1);
      insert into __CSV2SAS_dummy set _dummy_='';
  quit;

  %* Loop over all filenames ;
  %do csv2sas_i = 1 %to %sysfunc(countw(&filelist,%str(¤)));

  %* Current filename ;
  %let file = %scan(&filelist,&csv2sas_i,%str(¤));

  %* Assign a filename shortcut to the current file ;
  filename CSV_IN "&path.\&file.";

  %CreateSasFormats(&path.\&prefix.CODELISTS.csv);

  %ReadMetaDataFile(&path.\&prefix.Items.csv,__CSV2SAS_Items);

  %CreateMetaData(CSV_IN, %trim(%scan(%sysfunc(transtrn(&file.,&prefix.,%str())),-2,%str(.))));

  %CreateCommandsDataSet(%get_dataset_name(&file., &prefix., WORK));

  %runquit;

  data _null_;
    set __csv2sas_commands;
    put command;
  run;

  %* Read the data properly using an infile statement in a datastep ;;
  data _null_;
    set __CSV2SAS_commands;
    call execute(command);
  run;

  %* Deassign the filename shortcut ;;
  filename CSV_IN clear;

  %* Delete temporary datasets used if the current file seems OK ;
  proc delete data=__CSV2SAS_commands __CSV2SAS_meta __CSV2SAS_Items; run; quit;

  %PUT %STR(Viedoc data successfully imported to SAS using CSV2SAS version &__CSV2SAS_VERSION.);

  %end;

  proc delete data=__CSV2SAS_files __CSV2SAS_dummy;
  %runquit;

  %mend doWork;
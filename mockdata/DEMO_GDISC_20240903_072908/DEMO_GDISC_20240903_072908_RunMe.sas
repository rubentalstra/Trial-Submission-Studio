%macro grabpath;
  %qsubstr(%sysget(SAS_EXECFILEPATH),1,%length(%sysget(SAS_EXECFILEPATH))-%length(%sysget(SAS_EXECFILEname)))
%mend grabpath;
%let path=%grabpath;
%let prefix=DEMO_GDISC_20240903_072908_;
%include "&path.\CSV2SAS.sas";
%doWork(&path.,&prefix.);

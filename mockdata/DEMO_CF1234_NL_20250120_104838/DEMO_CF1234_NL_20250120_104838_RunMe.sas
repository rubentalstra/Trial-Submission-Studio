%macro grabpath;
  %qsubstr(%sysget(SAS_EXECFILEPATH),1,%length(%sysget(SAS_EXECFILEPATH))-%length(%sysget(SAS_EXECFILEname)))
%mend grabpath;
%let path=%grabpath;
%let prefix=DEMO_CF1234_NL_20250120_104838_;
%include "&path.\CSV2SAS.sas";
%doWork(&path.,&prefix.);

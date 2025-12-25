# Chapter 7: TRIAL DESIGN MODEL DATASETS

> Source pages 382–426 in `SDTMIG_v3.4.pdf`.


## Page 382

7 Trial Design Model Datasets
7.1 Introduction to Trial Design Model Datasets
7.1.1 Purpose of the Trial Design Model
ICH E3, Guidance for Industry, Structure and Content of Clinical Study Reports (available at http://www.ich.org/products/guidelines/), Section 9.1, calls for a brief, clear description of the overall plan and design of the study, and supplies examples of charts and diagrams for this purpose in Annex IIIa and Annex IIIb. Each Annex corresponds to an example trial, and each shows a diagram describing the study design and a table showing the schedule of assessments. The Trial Design Model provides a standardized way to describe those aspects of the planned conduct of a clinical trial shown in the study design diagrams of these examples. The standard Trial Design Datasets will allow reviewers to:
• Clearly and quickly grasp the design of a clinical trial
• Compare the designs of different trials
• Search a data warehouse for clinical trials with certain features
• Compare planned and actual treatments and visits for subjects in a clinical trial
Modeling a clinical trial in this standardized way requires the explicit statement of certain decision rules that may not be addressed or may be vague or ambiguous in the usual prose protocol document. Prospective modeling of the design of a clinical trial should lead to a clearer, better protocol. Retrospective modeling of the design of a clinical trial should ensure a clear description of how the trial protocol was interpreted by the sponsor.
7.1.2 Definitions of Trial Design Concepts
A clinical trial is a scientific experiment involving human subjects, intended to address certain scientific questions (i.e., the objectives of the trial). See the CDISC Glossary (https://www.cdisc.org/standards/glossary) for more complete definitions of clinical trial and objective.

| Concept | Definition |
| --- | --- |
| Trial design | The design of a clinical trial is a plan for what will be done to subjects and what data will be collected<br>about them, in the course of the trial, to address the trial's objectives. |
| Epoch | As part of the design of a trial, the planned period of subjects' participation in the trial is divided into<br>epochs. Each epoch is a period of time that serves a purpose in the trial as a whole. That purpose will<br>be at the level of the primary objectives of the trial. Typically, the purpose of an epoch will be to<br>expose subjects to a treatment or to prepare for such a treatment period (e.g., determine subject<br>eligibility, washout previous treatments), or to gather data on subjects after a treatment has ended.<br>Note that at this high level, a “treatment” is a treatment strategy, which may be simple (e.g., exposure<br>to a single drug at a single dose) or complex. Complex treatment strategies could involve tapering<br>through several doses, titrating dose according to clinical criteria, complex regimens involving multiple<br>drugs, or strategies for adding or dropping drugs according to clinical criteria. |
| Arm | An arm is a planned path through the trial. This path covers the entire time of the trial. The group of<br>subjects assigned to a planned path is also often colloquially called an "arm." The group of subjects<br>assigned to an arm is also often called a "treatment group"; in this sense, an arm is equivalent to a<br>treatment group. |
| Study cell | Each planned path through the trial (i.e., each arm) is divided into pieces, 1 for each epoch. Each of<br>these pieces is called a study cell. Thus, there is a study cell for each combination of arm and epoch.<br>Each study cell represents an implementation of the purpose of its associated epoch. For an epoch<br>whose purpose is to expose subjects to treatment, each study cell associated with the epoch has an<br>associated treatment strategy. For example, a 3-arm parallel trial might have a treatment epoch whose<br>purpose is to expose subjects to one of 3 study treatments: placebo, investigational product, or active<br>control. There would be 3 study cell associated with the treatment epoch, 1 for each arm. Each of<br>these study cells exposes the subject to 1 of the 3 study treatments. Another example involving more<br>complex treatment strategies would be a trial comparing the effects of cycles of chemotherapy drug A<br>given alone or in combination with drug B, where drug B is given as a pretreatment to each cycle of<br>drug A. |


## Page 383

7.1.3 Current and Future Contents of the Trial Design Model
Datasets currently in the Trial Design Model include:
• Trial Arms: Describes the sequences of elements in each epoch for each arm, and thus describes the
complete sequence of elements in each arm
• Trial Elements: Describes the elements used in the trial
• Trial Visits: Describes the planned schedule of visits
• Trial Disease Assessment: Provides information on the protocol-specified disease assessment schedule, and
is used for comparison with the actual occurrence of the efficacy assessments in order to determine whether there was good compliance with the schedule
• Trial Disease Milestones: Describes observations or activities identified for the trial which are anticipated
to occur in the course of the disease under study and which trigger the collection of data

| Concept | Definition |
| --- | --- |
| Element | An element is a basic building block in the trial design. It involves administering a planned intervention,<br>which may be treatment or no treatment, during a period of time. Elements for which the planned<br>intervention is "no treatment" would include elements for screening, washout, and follow-up. |
| Study cells and elements | Many (perhaps most) clinical trials involve a single, simple administration of a planned intervention<br>within a study cell. For some trials, however, the treatment strategy associated with a study cell<br>involves a complex series of administrations of treatment. In such cases it may be important to track<br>the component steps in a treatment strategy operationally; secondary objectives and safety analyses<br>also might require that data be grouped by the treatment step during which it was collected. The steps<br>within a treatment strategy may involve different doses of drug, different drugs, or different kinds of<br>care (e.g., preoperative, operative, and post-operative periods surrounding surgery). When the<br>treatment strategy for a study cell is simple, the study cell will contain a single element, and for many<br>purposes there is little value in distinguishing between the study cell and the element. However, when<br>the treatment strategy for a study cell consists of a complex series of treatments, a study cell can<br>contain multiple elements. There may be a fixed sequence of elements, or a repeating cycle of<br>elements, or some other complex pattern. In these cases, the distinction between a study cell and an<br>element is very useful. |
| Branch | In a trial with multiple arms, the protocol plans for each subject to be assigned to 1 arm. The time<br>within the trial at which this assignment takes place is the point at which the arm paths of the trial<br>diverge, and so is called a branch point. For many trials, the assignment to an arm happens all at one<br>time, so the trial has 1 branch point. For other trials, there may be 2 or more branches that collectively<br>assign a subject to an arm. The process that makes this assignment may be a randomization, but it<br>need not be. |
| Treatments | The word "treatment" may be used in connection with epochs, study cells, or elements, but has<br>somewhat different meanings in each context:<br>• Because epochs cut across arms, an epoch treatment is at a high level that does not specify<br>anything that differs between arms. For example, in a 3-period crossover study of 3 doses of<br>drug X, each treatment epoch is associated with drug X, but not with a specific dose.<br>• A study cell treatment is specific to a particular arm. For example, a parallel trial might have<br>study cell treatments placebo and drug X, without any additional detail (e.g., dose, frequency,<br>route of administration) being specified. A study cell treatment is at a relatively high level, the<br>level at which treatments might be planned in an early conceptual draft of the trial, or in the title<br>or objectives of the trial.<br>• An element treatment may be fairly detailed. For example, for an element representing a cycle<br>of chemotherapy, element treatment might specify 5 daily 100 mg doses of drug X.<br>The distinctions between these levels are not rigid, and depend on the objectives of the trial. For<br>example, route is generally a detail of dosing, but in a bioequivalence trial comparing IV and oral<br>administration of drug X, route is clearly part of study cell treatment. |
| Visit | The notion of a visit—a clinical encounter—derives from trials with outpatients, where subjects interact<br>with the investigator during visits to the investigator's clinical site. However, the term is used in other<br>trials, where a trial visit may not correspond to a physical visit. For example, in a trial with inpatients,<br>time may be subdivided into visits, even though subjects are in hospital throughout the trial. For<br>example, data for a screening visit may be collected over the course of more than 1 physical visit. One<br>of the main purposes of visits is the performance of assessments, but not all assessments need take<br>place at clinic visits; some assessments may be performed by means of telephone contacts, electronic<br>devices, or call-in systems. The protocol should specify what contacts are considered visits and how<br>they are defined. |


## Page 384

• Trial Inclusion/Exclusion Criteria: Describes the criteria used to screen subjects
• Trial Summary: Lists key facts (parameters) about the trial that are likely to appear in a registry of clinical
trials The Trial Inclusion/Exclusion Criteria (TI) dataset is discussed in Section 7.4.1, Trial Inclusion/Exclusion Criteria. The Inclusion/Exclusion Criteria Not Met (IE) domain described in Section 6.3.4, Inclusion/Exclusion Criteria Not Met, contains the actual exceptions to those criteria for enrolled subjects. The current Trial Design Model has limitations in representing protocols, which include:
• Plans for indefinite numbers of repeating elements (e.g., indefinite numbers of chemotherapy cycles)
• Indefinite numbers of visits (e.g., periodic follow-up visits for survival)
• Indefinite numbers of epochs
• Indefinite numbers of arms
The last 2 situations arise in dose-escalation studies where increasing doses are given until stopping criteria are met. Some dose-escalation studies enroll a new cohort of subjects for each new dose, and so, at the planning stage, have an indefinite number of arms. Other dose-escalation studies give new doses to a continuing group of subjects, and so are planned with an indefinite number of epochs. There may also be limitations in representing other patterns of Elements within a Study Cell that are more complex than a simple sequence. For the purpose of submissions about trials that have already completed, these limitations are not critical, so it is expected that development of the Trial Design Model to address these limitations will have a minimal impact on the SDTM.
7.2 Experimental Design (TA and TE)
This subsection contains the Trial Design datasets that describe the planned design of the study, and provide the representation of study treatment in its most granular components (Section 7.2.2, Trial Elements (TE)), as well as the representation of all sequences of these components (Section 7.2.1, Trial Arms (TA)) as specified by the study protocol. The TA and TE datasets are interrelated, and they provide the building blocks for the development of subject-level treatment information (see Sections 5.2, Demographics (DM), and 5.3, Subject Elements (SE), for the subject’s actual study treatment information).
7.2.1 Trial Arms (TA)
TA – Description/Overview A trial design domain that contains each planned arm in the trial. This section contains:
• The Trial Arms dataset and assumptions
• A series of example trials, which illustrate the development of the TA dataset
• Advice on various issues in the development of the TA dataset
• A recap of the TA dataset and the function of its variables
TA – Specification ta.xpt, Trial Arms — Trial Design. One record per planned Element per Arm, Tabulation.

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | TA | Identifier | Two-character abbreviation for the domain. | Req |
| ARMCD | Planned Arm<br>Code | Char | * | Topic | ARMCD is limited to 20 characters and does not have special | Req |
|  |  |  |  |  | character restrictions. The maximum length of ARMCD is longer |  |


## Page 385

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TA – Assumptions
1. TAETORD is an integer. In general, the value of TAETORD is 1 for the first element in each arm, 2 for the
second element in each arm, and so on. Occasionally, it may be convenient to skip some values (see Example Trial 6). Although the values of TAETORD need not always be sequential, their order must always be the correct order for the elements in the arm path.
2. Elements in different arms with the same value of TAETORD may or may not be at the same time,
depending on the design of the trial. The example trials illustrate a variety of possible situations. The same element may occur more than once within an arm.
3. TABRANCH describes the outcome of a branch decision point in the trial design for subjects in the arm. A
branch decision point takes place between epochs, and is associated with the element that ends at the decision point. For instance, if subjects are assigned to an arm where they receive treatment A through a randomization at the end of element X, the value of TABRANCH for element X would be "Randomized to A."
4. Branch decision points may be based on decision processes other than randomizations (e.g., clinical
evaluations of disease response, subject choice).
5. There is usually some gap in time between the performance of a randomization and the start of randomized
treatment. However, in many trials this gap in time is small and it is highly unlikely that subjects will leave the trial between randomization and treatment. In these circumstances, the trial does not need to be modeled with this time period between randomization and start of treatment as a separate element.
6. Some trials include multiple paths that are closely enough related so that they are all considered to belong
to 1 arm. In general, this set of paths will include a "complete" path along with shorter paths that skip some elements. The sequence of elements represented in the trial arms should be the complete, longest path. TATRANS describes the decision points that may lead to a shortened path within the arm.
7. If an element does not end with a decision that could lead to a shortened path within the arm, then
TATRANS will be blank. If there is such a decision, TATRANS will be in a form like, "If condition X is true, then go to epoch Y" or "If condition X is true, then go to element with TAETORD = 'Z'".

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
|  |  |  |  |  | than that for other "short" variables to accommodate the kind of |  |
|  |  |  |  |  | values that are likely to be needed for crossover trials. For |  |
|  |  |  |  |  | example, if ARMCD values for a 7-period crossover were |  |
|  |  |  |  |  | constructed using 2-character abbreviations for each treatment |  |
|  |  |  |  |  | and separating hyphens, the length of ARMCD values would be |  |
|  |  |  |  |  | 20. |  |
| ARM | Description of<br>Planned Arm | Char | * | Synonym<br>Qualifier | Name given to an arm or treatment group. | Req |
| TAETORD | Planned Order<br>of Element<br>within Arm | Num |  | Timing | Number that gives the order of the element within the arm. | Req |
| ETCD | Element Code | Char | * | Record<br>Qualifier | ETCD (the companion to ELEMENT) is limited to 8 characters | Req |
|  |  |  |  |  | and does not have special character restrictions. These values |  |
|  |  |  |  |  | should be short for ease of use in programming, but it is not |  |
|  |  |  |  |  | expected that ETCD will need to serve as a variable name. |  |
| ELEMENT | Description of<br>Element | Char | * | Synonym<br>Qualifier | The name of the element. The same element may occur more | Perm |
|  |  |  |  |  | than once within an arm. |  |
| TABRANCH | Branch | Char |  | Rule | Condition subject met, at a "branch" in the trial design at the end | Exp |
|  |  |  |  |  | of this element, to be included in this arm (e.g., "Randomization |  |
|  |  |  |  |  | to DRUG X"). |  |
| TATRANS | Transition Rule | Char |  | Rule | If the trial design allows a subject to transition to an element | Exp |
|  |  |  |  |  | other than the next element in sequence, then the conditions for |  |
|  |  |  |  |  | transitioning to those other elements, and the alternative |  |
|  |  |  |  |  | element sequences, are specified in this rule (e.g., "Responders |  |
|  |  |  |  |  | go to washout"). |  |
| EPOCH | Epoch | Char | (EPOCH) | Timing | Name of the trial epoch with which this element of the arm is | Req |
|  |  |  |  |  | associated. |  |


## Page 386

8. EPOCH is not strictly necessary for describing the sequence of elements in an arm path, but it is the
conceptual basis for comparisons between arms and also provides a useful way to talk about what is happening in a blinded trial while it is blinded. During periods of blinded treatment, blinded participants will not know which arm and element a subject is in, but EPOCH should provide a description of the time period that does not depend on knowing arm.
9. EPOCH should be assigned in such a way that elements from different arms with the same value of
EPOCH are "comparable" in some sense. The degree of similarity across arms varies considerably in different trials, as illustrated in the examples.
10. EPOCH values for multiple similar epochs:
a. When a study design includes multiple epochs with the same purpose (e.g., multiple similar treatment epochs), it is recommended that the EPOCH values be terms from controlled terminology, but with numbers appended. For example, multiple treatment epochs could be represented using "TREATMENT 1", "TREATMENT 2", and so on. Because the codelist is extensible, this convention allows multiple similar epochs to be represented without adding numbered terms to the CDISC Controlled Terminology for epoch. The inclusion of multiple numbered terms in the EPOCH codelist is not considered to add value. b. Note that the controlled terminology does include some more granular terms for distinguishing between epochs that differ in ways other than mere order, and these terms should be used where applicable, as they are more informative. For example, when "BLINDED TREATMENT" and "OPEN LABEL TREATMENT" are applicable, those terms would be preferred over "TREATMENT 1" and "TREATMENT 2".
11. Note that study cells are not explicitly defined in the TA dataset. A set of records with a common value of
both ARMCD and EPOCH constitute the description of a study cell. Transition rules within this set of records are also part of the description of the study cell.
12. EPOCH may be used as a timing variable in other datasets, such as Exposure (EX) and Disposition (DS),
and values of EPOCH must be different for different epochs. For instance, in a crossover trial with 3 treatment epochs, each must be given a distinct name; all 3 cannot be called “TREATMENT”. TA – Examples The core of the Trial Design Model is the TA dataset. For each arm of the trial, the TA dataset contains 1 record for each occurrence of an element in the path of the arm. Although the TA dataset has 1 record for each trial element traversed by subjects assigned to the arm, it is generally more useful to work out the overall design of the trial at the study cell level first, then to work out the elements within each study cell, and finally to develop the definitions of the elements that are contained in the Trial Elements (TE) table. When working out the design of a trial, it is generally useful to draw diagrams such as those mentioned in ICH E3. The protocol may include a diagram that can serve as a starting point. Such a diagram can then be converted into a trial design matrix that displays the study cells and which in turn can be converted into the TA dataset. This section uses example trials of increasing complexity to illustrate the concepts of trial design. For each example trial, the process of working out the TA table is illustrated by means of a series of diagrams and tables, including:
• A diagram showing the branching structure of the trial in a “study schema” format such as might appear in
a protocol
• A diagram that shows the “prospective” view of the trial (i.e., the view of those participating in the trial).
This is similar to the study schema view in that it usually shows a single pool of subjects at the beginning of the trial, with the pool of subjects being split into separate treatment groups at randomizations and other branches. Such diagrams include the epochs of the trial, and, for each group of subjects and each epoch, the sequence of elements within each epoch for that treatment group. The arms are also indicated on these diagrams.
• A diagram that shows the “retrospective” view of the trial (i.e., the view of the analyst reporting on the
trial). This style of diagram looks more like a matrix; it is also more like the structure of the TA dataset. The retrospective view is arm-centered and shows, for each study cell (epoch/arm combination) the


## Page 387

sequence of elements within that study cell. It can be thought of as showing, for each arm, the elements traversed by a subject who completed that arm as intended.
• If the trial is blinded, a diagram that shows the trial as it appears to a blinded participant
• A trial design matrix, an alternative format for representing most of the information in the diagram that
shows arms and epochs, and which emphasizes the study cells
• The TA dataset
Example 1 should be reviewed before reading other examples, as it explains the conventions used for all diagrams and tables in the examples. Example 1 Diagrams that represent study schemas generally conceive of time as moving from left to right, using horizontal lines to represent periods of time and slanting lines to represent branches into separate treatments, convergence into a common follow-up, or crossover to a different treatment. In this type of document, diagrams are drawn using "blocks" corresponding to trial elements rather than horizontal lines. Trial elements are the various treatment and non-treatment time periods of the trial and we want to emphasize the separate trial elements might otherwise be "hidden" in a single horizontal line. See Section 7.2.2, Trial Elements (TE), for more information about defining trial elements. In general, the elements of a trial will be fairly clear. However, in the process of working out a trial design, alternative definitions of trial elements may be considered, in which case diagrams for each alternative may be constructed. In the study schema diagrams in this example, the only slanting lines used are those that represent branches (i.e., decision points where subjects are divided into separate treatment groups). One advantage of this style of diagram, which does not show convergence of separate paths into a single block, is that the number of arms in the trial can be determined by counting the number of parallel paths at the right end of the diagram. As illustrated in the study schema diagram for Example Trial 1, this simple parallel trial has 3 arms, corresponding to the 3 possible left-to-right "paths" through the trial. Each path corresponds to 1 of the 3 treatment elements at the right end of the diagram. Randomization is represented by the 3 red arrows leading from the Run-in block. Example Trial 1, Parallel Design Study Schema The next diagram for this trial shows the 3 epochs of the trial, indicates the 3 arms, and shows the sequence of elements for each group of subjects in each epoch. The arrows are at the right side of the diagram because it is at the end of the trial that all the separate paths through the trial can be seen. Note that, in this diagram, randomization— which was shown using 3 red arrows connecting the Run-in block with the 3 treatment blocks in the first diagram— is indicated by a note with an arrow pointing to the line between 2 epochs.


## Page 388

Example Trial 1, Parallel Design Prospective View The next diagram can be thought of as the retrospective view of a trial, the view back from a point in time when a subject’s assignment to an arm is known. In this view, the trial appears as a grid, with an arm represented by a series of study cells, one for each epoch, and a sequence of elements within each study cell. In this example (as in many trials), there is exactly 1 element in each study cell. Later examples will illustrate that this is not always the case. Example Trial 1, Parallel Design Retrospective View The next diagram shows the trial from the viewpoint of blinded participants. To blinded participants in this trial, all arms look alike. They know when a subject is in the screen element or the run-in element, but when a subject is in the treatment epoch, participants know only that the subject is receiving a study drug, not which study drug, and therefore not which element. Example Trial 1, Parallel Design Blinded View


## Page 389

A trial design matrix is a table with a row for each arm in the trial and a column for each epoch in the trial. It is closely related to the retrospective view of the trial, and many users may find it easier to construct a table than to draw a diagram. The cells in the matrix represent the study cells, which are populated with trial elements. In this trial, each study cell contains exactly 1 element. As illustrated in the following table, the columns of a trial design matrix are the epochs of the trial, the rows are the arms of the trial, and the cells of the matrix (the study cells) contain elements. Note that randomization is not represented in the trial design matrix. All of the preceding diagrams and the trial design matrix are alternative representations of the trial design. None of them contains all the information that will be in the finished TA dataset; users may find it useful to draw some or all of these diagrams when working out the dataset. Trial Design Matrix

For Example Trial 1, the conversion of the trial design matrix into the TA dataset is straightforward. For each cell of the matrix, there is a record in the TA dataset. ARM, EPOCH, and ELEMENT can be populated directly from the matrix. TAETORD acts as a sequence number for the elements within an arm, so it can be populated by counting across the cells in the matrix. The randomization information, which is not represented in the trial design matrix, is held in TABRANCH in the TA dataset. TABRANCH is populated only if there is a branch at the end of an element for the arm. When TABRANCH is populated, it describes how the decision at the branch point would result in a subject being in this arm. ta.xpt

|  | Screen | Run-in | Treatmen |
| --- | --- | --- | --- |
| Placebo | Screen | Run-in | PLACEBO |
| A | Screen | Run-in | DRUG A |
| B | Screen | Run-in | DRUG B |

Example 2 The following diagram for a crossover trial does not use the crossing slanted lines sometimes used to represent crossover trials, because the order of the blocks is sufficient to represent the design of the trial. Slanted lines are used only to represent the branch point at randomization, when a subject is assigned to a sequence of treatments. As in most crossover trials, the arms are distinguished by the order of treatments, with the same treatments present in each arm. Note that even though all 3 arms of this trial end with the same block (i.e., the block for the follow-up element), the diagram does not show the arms converging into one block. Also note that the same block (the “rest” element) occurs twice within each arm. Elements are conceived of as “reusable” and can appear in more than 1 arm, in more than 1 epoch, and more than once in an arm. Example Trial 2, Crossover Trial Study Schema

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX1 | TA | P | Placebo | 1 | SCRN | Screen |  |  | SCREENING |
| 2 | EX1 | TA | P | Placebo | 2 | RI | Run-In | Randomized to Placebo |  | RUN-IN |
| 3 | EX1 | TA | P | Placebo | 3 | P | Placebo |  |  | TREATMENT |
| 4 | EX1 | TA | A | A | 1 | SCRN | Screen |  |  | SCREENING |
| 5 | EX1 | TA | A | A | 2 | RI | Run-In | Randomized to Drug A |  | RUN-IN |
| 6 | EX1 | TA | A | A | 3 | A | Drug A |  |  | TREATMENT |
| 7 | EX1 | TA | B | B | 1 | SCRN | Screen |  |  | SCREENING |
| 8 | EX1 | TA | B | B | 2 | RI | Run-In | Randomized to Drug B |  | RUN-IN |
| 9 | EX1 | TA | B | B | 3 | B | Drug B |  |  | TREATMENT |


## Page 390

The next diagram for this crossover trial shows the prospective view of the trial; it identifies the epoch and arms of the trial, and gives each a name. As for most crossover studies, the objectives of the trial will be addressed by comparisons between the arms and by within-subject comparisons between treatments. Because the design depends on differentiating the periods during which the subject receives the 3 different treatments, there are 3 different treatment epochs. The fact that the rest periods are identified as separate epochs suggests that these also play an important part in the design of the trial; they are probably designed to allow subjects to return to “baseline,” with data collected to show that this occurred. Note that epochs are not considered reusable; each epoch has a different name, even though all the treatment epochs are similar and both the rest epochs are similar. As with the first example trial, there is a one-to-one relationship between the epochs of the trial and the elements in each arm. Example Trial 2, Crossover Trial Prospective View The next diagram shows the retrospective view of the trial. Example Trial 2, Crossover Trial Retrospective View The last diagram for this trial shows the trial from the viewpoint of blinded participants. As in the simple parallel trial in Example Trial 1, blinded participants see only 1 sequence of elements; during the treatment epochs they do not know which of the treatment elements a subject is in.


## Page 391

Example Trial 2, Crossover Trial Blinded View The following table illustrates the trial design matrix for this crossover example trial. It corresponds closely to the preceding retrospective diagram. Trial Design Matrix

It is straightforward to produce the TA dataset for this crossover trial from the diagram showing arms and epochs, or from the trial design matrix. ta.xpt

|  | Screen | First Treatment | First Rest S | econd Treatment | Second Rest | Third Treatment F |
| --- | --- | --- | --- | --- | --- | --- |
| P-5-1 | 0 Screen | Placebo | Rest 5 | mg | Rest | 10 mg F |
| 5-P-1 | 0 Screen | 5 mg | Rest P | lacebo | Rest | 10 mg F |
| 5-10- | P Screen | 5 mg | Rest 1 | 0 mg | Rest | Placebo F |

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 1 | SCRN | Screen | Randomized to Placebo - 5<br>mg - 10 mg |  | SCREENING |
| 2 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 2 | P | Placebo |  |  | TREATMENT<br>1 |
| 3 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 3 | REST | Rest |  |  | WASHOUT 1 |
| 4 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 4 | 5 | 5 mg |  |  | TREATMENT<br>2 |
| 5 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 5 | REST | Rest |  |  | WASHOUT 2 |
| 6 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 6 | 10 | 10 mg |  |  | TREATMENT<br>3 |
| 7 | EX2 | TA | P-5-10 | Placebo-5mg-<br>10mg | 7 | FU | Follow-up |  |  | FOLLOW-UP |
| 8 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 1 | SCRN | Screen | Randomized to 5 mg -<br>Placebo - 10 mg |  | SCREENING |
| 9 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 2 | 5 | 5 mg |  |  | TREATMENT<br>1 |
| 10 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 3 | REST | Rest |  |  | WASHOUT 1 |
| 11 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 4 | P | Placebo |  |  | TREATMENT<br>2 |
| 12 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 5 | REST | Rest |  |  | WASHOUT 2 |
| 13 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 6 | 10 | 10 mg |  |  | TREATMENT<br>3 |
| 14 | EX2 | TA | 5-P-10 | 5mg-Placebo-<br>10mg | 7 | FU | Follow-up |  |  | FOLLOW-UP |
| 15 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 1 | SCRN | Screen | Randomized to 5 mg - 10<br>mg – Placebo |  | SCREENING |
| 16 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 2 | 5 | 5 mg |  |  | TREATMENT<br>1 |
| 17 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 3 | REST | Rest |  |  | WASHOUT 1 |
| 18 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 4 | 10 | 10 mg |  |  | TREATMENT<br>2 |
| 19 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 5 | REST | Rest |  |  | WASHOUT 2 |


## Page 392

Example 3 Each of the paths for the trial illustrated in the following diagram goes through one branch point at randomization, and then through another branch point when response is evaluated. This results in 4 arms, corresponding to the number of possible paths through the trial, and also to the number of blocks at the right end of the diagram. The fact that there are only 2 kinds of block at the right end (Open DRUG X and Rescue) does not affect the fact that there are 4 paths and thus 4 arms. Example Trial 3, Multiple Branches Study Schema The next diagram for this trial is the prospective view. It shows the epochs of the trial and how the initial group of subjects is split into 2 treatment groups for the double-blind treatment epoch, and how each of those initial treatment groups is split in 2 at the response evaluation, resulting in the 4 arms of this trial The names of the arms have been chosen to represent the outcomes of the successive branches that, together, assign subjects to arms. These compound names were chosen to facilitate description of subjects who may drop out of the trial after the first branch and before the second branch. Example 7 in Section 5.2, Demographics, illustrates DM and Subject Elements (SE) data for such subjects. Example Trial 3, Multiple Branches Prospective View

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 6 | P | Placebo |  |  | TREATMENT<br>3 |
| 21 | EX2 | TA | 5-10-P | 5mg-10mg-<br>Placebo | 7 | FU | Follow-up |  |  | FOLLOW-UP |


## Page 393

The next diagram shows the retrospective view. As with the first 2 example trials, there is 1 element in each study cell. Example Trial 3, Multiple Branches Retrospective View The last diagram for this trial shows the trial from the viewpoint of blinded participants. Since the prospective view is the view most relevant to study participants, the blinded view shown here is a prospective view. Because blinded participants can tell which treatment a subject receives in the Open Label epoch, they see 2 possible element sequences. Example Trial 3, Multiple Branches Blinded View The trial design matrix for this trial can be constructed easily from the diagram showing arms and epochs. Trial Design Matrix

|  | Screen | Double Blind | Open Label |
| --- | --- | --- | --- |
| A-Open A | Screen | Treatment A | Open Drug A |
| A-Rescue | Screen | Treatment A | Rescue |
| B-Open A | Screen | Treatment B | Open Drug A |
| B-Rescue | Screen | Treatment B | Rescue |


## Page 394

Creating the TA dataset for this example trial is similarly straightforward. Note that because there are 2 branch points in this trial, TABRANCH is populated for 2 records in each arm. Note also that the values of ARMCD, like the values of ARM, reflect the 2 separate processes that result in a subject's assignment to an arm. ta.xpt

See Section 7.2.1.1 Trial Arms Issues, Distinguishing Between Branches and Transitions, for additional discussion regarding when a decision point in a trial design should be considered to give rise to a new arm. Example 4 The following diagram uses a new symbol, a large curved arrow representing the fact that the chemotherapy treatment (A or B) and the rest period that follows it are to be repeated. In this trial, the chemotherapy cycles are to be repeated until disease progression. Although some chemotherapy trials specify a maximum number of cycles, protocols that allow an indefinite number of repeats are not uncommon. Example Trial 4, Cyclical Chemotherapy Study Schema The next diagram shows the prospective view of this trial. Note that, in spite of the repeating element structure, this is, at its core, a 2-arm parallel study, and thus has 2 arms. In SDTMIG 3.1.1, there was an implicit assumption that each element must be in a separate epoch, and trials with cyclical chemotherapy were difficult to handle. The introduction of the concept of study cells and the dropping of the assumption that elements and epochs have a oneto-one relationship resolved these difficulties. This trial is best treated as having just 3 epochs, since the main objectives of the trial involve comparisons between the 2 treatments and do not require data to be considered cycle by cycle.

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX3 | TA | AA | A-Open<br>A | 1 | SCRN | Screen | Randomized to Treatment A |  | SCREENING |
| 2 | EX3 | TA | AA | A-Open<br>A | 2 | DBA | Treatment<br>A | Assigned to Open Drug A on<br>basis of response evaluation |  | BLINDED<br>TREATMENT |
| 3 | EX3 | TA | AA | A-Open<br>A | 3 | OA | Open Drug<br>A |  |  | OPEN LABEL<br>TREATMENT |
| 4 | EX3 | TA | AR | A-<br>Rescue | 1 | SCRN | Screen | Randomized to Treatment A |  | SCREENING |
| 5 | EX3 | TA | AR | A-<br>Rescue | 2 | DBA | Treatment<br>A | Assigned to Rescue on basis<br>of response evaluation |  | BLINDED<br>TREATMENT |
| 6 | EX3 | TA | AR | A-<br>Rescue | 3 | RSC | Rescue |  |  | OPEN LABEL<br>TREATMENT |
| 7 | EX3 | TA | BA | B-Open<br>A | 1 | SCRN | Screen | Randomized to Treatment B |  | SCREENING |
| 8 | EX3 | TA | BA | B-Open<br>A | 2 | DBB | Treatment<br>B | Assigned to Open Drug A on<br>basis of response evaluation |  | BLINDED<br>TREATMENT |
| 9 | EX3 | TA | BA | B-Open<br>A | 3 | OA | Open Drug<br>A |  |  | OPEN LABEL<br>TREATMENT |
| 10 | EX3 | TA | BR | B-<br>Rescue | 1 | SCRN | Screen | Randomized to Treatment B |  | SCREENING |
| 11 | EX3 | TA | BR | B-<br>Rescue | 2 | DBB | Treatment<br>B | Assigned to Rescue on basis<br>of response evaluation |  | BLINDED<br>TREATMENT |
| 12 | EX3 | TA | BR | B-<br>Rescue | 3 | RSC | Rescue |  |  | OPEN LABEL<br>TREATMENT |


## Page 395

Example Trial 4, Cyclical Chemotherapy Prospective View The next diagram shows the retrospective view of this trial. Example Trial 4, Cyclical Chemotherapy Retrospective View For the purpose of developing a TA dataset for this oncology trial, the diagram must be redrawn to explicitly represent multiple treatment and rest elements. If a maximum number of cycles is not given by the protocol, then— for the purposes of constructing an SDTM TA dataset for submission, which can only take place after the trial is complete—the number of repeats included in the TA dataset should be the maximum number of repeats that occurred in the trial. The next diagram assumes that the maximum number of cycles that occurred in this trial was 4. Some subjects will not have received all 4 cycles, because their disease progressed. The rule that directed that they receive no further cycles of chemotherapy is represented by a set of green arrows, 1 at the end of each rest epoch, that shows that a subject “skips forward” if their disease progresses. In the TA dataset, each skip-forward instruction is a transition rule, recorded in the TATRANS variable; when TATRANS is not populated, the rule is to transition to the next element in sequence. Example Trial 4, Cyclical Chemotherapy Retrospective View with Explicit Repeats


## Page 396

The logistics of dosing mean that few oncology trials are blinded; the next diagram, however, shows the trial from the viewpoint of blinded participants if this trial is blinded. Example Trial 4, Cyclical Chemotherapy Blinded View The trial design matrix for this example trial corresponds to the diagram showing the retrospective view, with explicit repeats of the treatment and rest elements. As previously noted, the trial design matrix does not include information regarding when randomization occurs; similarly, information corresponding to the skip-forward rules is not represented in the trial design matrix. Trial Design Matrix

The TA dataset for this example trial requires the use of the TATRANS variable to represent the "repeat until disease progression" feature (the green "skip forward" arrow represented this rule in the diagrams). In the TA dataset, TATRANS is populated for each element with a green arrow in the diagram. In other words, if there is a possibility that a subject will, at the end of this element, skip forward to a later part of the arm, then TATRANS is populated with the rule describing the conditions under which a subject will go to a later element. If the subject always goes to the next element in the arm (see Example Trials 1-3), then TATRANS is null. The TA dataset presented below corresponds to the trial design matrix. ta.xpt

| Screen | Treatment |  |  |  |  |  |  |  | Follow-up |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A Screen | Trt A | Rest | Trt A | Rest | Trt A | Rest | Trt A R | est | Follow-up |
| B Screen | Trt B | Rest | Trt B | Rest | Trt B | Rest | Trt B R | est | Follow-up |

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX4 | TA | A | A | 1 | SCRN | Screen | Randomized to<br>A |  | SCREENING |
| 2 | EX4 | TA | A | A | 2 | A | Trt A |  |  | TREATMENT |
| 3 | EX4 | TA | A | A | 3 | REST | Rest |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 4 | EX4 | TA | A | A | 4 | A | Trt A |  |  | TREATMENT |
| 5 | EX4 | TA | A | A | 5 | REST | Rest |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 6 | EX4 | TA | A | A | 6 | A | Trt A |  |  | TREATMENT |
| 7 | EX4 | TA | A | A | 7 | REST | Rest |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 8 | EX4 | TA | A | A | 8 | A | Trt A |  |  | TREATMENT |
| 9 | EX4 | TA | A | A | 9 | REST | Rest |  |  | TREATMENT |
| 10 | EX4 | TA | A | A | 10 | FU | Follow-up |  |  | FOLLOW-UP |
| 11 | EX4 | TA | B | B | 1 | SCRN | Screen | Randomized to<br>B |  | SCREENING |
| 12 | EX4 | TA | B | B | 2 | B | Trt B |  |  | TREATMENT |
| 13 | EX4 | TA | B | B | 3 | REST | Rest |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 14 | EX4 | TA | B | B | 4 | B | Trt B |  |  | TREATMENT |
| 15 | EX4 | TA | B | B | 5 | REST | Rest |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 16 | EX4 | TA | B | B | 6 | B | Trt B |  |  | TREATMENT |
| 17 | EX4 | TA | B | B | 7 | REST | Rest |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 18 | EX4 | TA | B | B | 8 | B | Trt B |  |  | TREATMENT |
| 19 | EX4 | TA | B | B | 9 | REST | Rest |  |  | TREATMENT |
| 20 | EX4 | TA | B | B | 10 | FU | Follow-up |  |  | FOLLOW-UP |


## Page 397

Example 5 Example Trial 5 is much like Example Trial 4, in that the 2 treatments being compared are given in cycles, and the total length of the cycle is the same for both treatments. In this trial, however, treatment A is given over longer duration than treatment B. Because of this difference in treatment patterns, this trial cannot be blinded. Example Trial 5, Different Chemo Durations Study Schema The assumption of a one-to-one relationship between elements and epochs makes such situations difficult to handle. However, without that assumption, this trial is essentially the same as Trial 4. The next diagram shows the retrospective view of this trial. Example Trial 5, Cyclical Chemotherapy Retrospective View The trial design matrix for this trial is almost the same as for Example Trial 4; the only difference is that the maximum number of cycles for this trial was assumed to be 3. Trial Design Matrix

The TA dataset for this trial shown below corresponds to the trial design matrix. ta.xpt

| Screen | Treatment F |  |  |  |  |  |
| --- | --- | --- | --- | --- | --- | --- |
| Screen | Trt A | Rest A | Trt A | Rest A | Trt A | Rest A F |
| Screen | Trt B | Rest B | Trt B | Rest B | Trt B | Rest B F |

| ow STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| EX5 | TA | A | A | 1 | SCRN | Screen | Randomized to<br>A |  | SCREENING |
| EX5 | TA | A | A | 2 | A | Trt A |  |  | TREATMENT |
| EX5 | TA | A | A | 3 | RESTA | Rest A |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| EX5 | TA | A | A | 4 | A | Trt A |  |  | TREATMENT |
| EX5 | TA | A | A | 5 | RESTA | Rest A |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| EX5 | TA | A | A | 6 | A | Trt A |  |  | TREATMENT |
| EX5 | TA | A | A | 7 | RESTA | Rest A |  |  | TREATMENT |
| EX5 | TA | A | A | 8 | FU | Follow-up |  |  | FOLLOW-UP |
| EX5 | TA | B | B | 1 | SCRN | Screen | Randomized to<br>B |  | SCREENING |


## Page 398

Example 6 Example Trial 6 is an oncology trial comparing 2 types of chemotherapy that are given using cycles of different lengths with different internal patterns. Treatment A is given in 3-week cycles with a longer duration of treatment and a short rest; treatment B is given in 4-week cycles with a short duration of treatment and a long rest. Example Trial 6, Different Cycle Durations Study Schema The design of this trial is very similar to that for Example Trials 4 and 5. The main difference is that there are 2 different rest elements: the short one used with drug A and the long one used with drug B. The next diagram shows the retrospective view of this trial. Example Trial 6, Cyclical Chemotherapy Retrospective View The trial design matrix for this trial assumes that there was a maximum of 4 cycles of drug A and a maximum of three cycles of drug B. Trial Design Matrix

| ow STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 EX5 | TA | B | B | 2 | B | Trt B |  |  | TREATMENT |
| 1 EX5 | TA | B | B | 3 | RESTB | Rest B |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 2 EX5 | TA | B | B | 4 | B | Trt B |  |  | TREATMENT |
| 3 EX5 | TA | B | B | 5 | RESTB | Rest B |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 4 EX5 | TA | B | B | 6 | B | Trt B |  |  | TREATMENT |
| 5 EX5 | TA | B | B | 7 | RESTB | Rest B |  |  | TREATMENT |
| 6 EX5 | TA | B | B | 8 | FU | Follow-up |  |  | FOLLOW-UP |

In the following TA dataset, because the treatment epoch for arm A has more elements than the treatment epoch for arm B, TAETORD is 10 for the follow-up element in arm A, but 8 for the follow-up element in arm B. (It would also be possible to assign a TAETORD value of 10 to the follow-up element in arm B.) The primary purpose of TAETORD is to order elements within an arm; leaving gaps in the series of TAETORD values does not interfere with this purpose.

| Screen | Treatment |  |  |  |
| --- | --- | --- | --- | --- |
| Screen | Trt A | Rest A Trt A Rest A Trt A | Rest A | Trt A Rest A |
| Screen | Trt B | Rest B Trt B Rest B | Trt B | Rest B |


## Page 399

ta.xpt

Example 7 In open trials, there is no requirement to maintain a blind, and the arms of a trial may be quite different from each other. In such a case, changes in treatment in one arm may differ in number and timing from changes in treatment in another, so that there is nothing like a one-to-one match between the elements in the different arms. In such a case, epochs are likely to be defined as broad intervals of time, spanning several elements, and chosen to correspond to periods of time that will be compared in analyses of the trial. Example Trial 7, RTOG 93-09, involves treatment of lung cancer with chemotherapy and radiotherapy, with or without surgery. The protocol (RTOG-93-09), which was provided by the Radiation Oncology Therapy Group (RTOG), does not include a study schema diagram, but does include a text-based representation of diverging “options” to which a subject may be assigned. All subjects go through the branch point at randomization, when they are assigned to either chemotherapy plus radiotherapy (CR) or chemotherapy and radiotherapy plus surgery (CRS). All subjects receive induction chemotherapy and radiation, with a slight difference between those randomized to the
2 arms during the second cycle of chemotherapy. Those randomized to the non-surgery arm are evaluated for
disease somewhat earlier, to avoid delays in administering the radiation boost to those whose disease has not progressed. After induction chemotherapy and radiation, subjects are evaluated for disease progression, and those whose disease has progressed stop treatment, but enter follow-up. Not all subjects randomized to receive surgery who do not have disease progression will necessarily receive surgery. If they are poor candidates for surgery or do not wish to receive surgery, they will not receive surgery, but will receive further chemotherapy. The following diagram is based on the text “schema” in the protocol, with the 5 options it names. The diagram in this form might suggest that the trial has 5 arms.

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX6 | TA | A | A | 1 | SCRN | Screen | Randomized to<br>A |  | SCREENING |
| 2 | EX6 | TA | A | A | 2 | A | Trt A |  |  | TREATMENT |
| 3 | EX6 | TA | A | A | 3 | RESTA | Rest A |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 4 | EX6 | TA | A | A | 4 | A | Trt A |  |  | TREATMENT |
| 5 | EX6 | TA | A | A | 5 | RESTA | Rest A |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 6 | EX6 | TA | A | A | 6 | A | Trt A |  |  | TREATMENT |
| 7 | EX6 | TA | A | A | 7 | RESTA | Rest A |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 8 | EX6 | TA | A | A | 8 | A | Trt A |  |  | TREATMENT |
| 9 | EX6 | TA | A | A | 9 | RESTA | Rest A |  |  | TREATMENT |
| 10 | EX6 | TA | A | A | 10 | FU | Follow-up |  |  | FOLLOW-UP |
| 11 | EX6 | TA | B | B | 1 | SCRN | Screen | Randomized to<br>B |  | SCREENING |
| 12 | EX6 | TA | B | B | 2 | B | Trt B |  |  | TREATMENT |
| 13 | EX6 | TA | B | B | 3 | RESTB | Rest B |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 14 | EX6 | TA | B | B | 4 | B | Trt B |  |  | TREATMENT |
| 15 | EX6 | TA | B | B | 5 | RESTB | Rest B |  | If disease progression, go to<br>Follow-up Epoch | TREATMENT |
| 16 | EX6 | TA | B | B | 6 | B | Trt B |  |  | TREATMENT |
| 17 | EX6 | TA | B | B | 7 | RESTB | Rest B |  |  | TREATMENT |
| 18 | EX6 | TA | B | B | 8 | FU | Follow-up |  |  | FOLLOW-UP |


## Page 400

Example Trial 7, RTOG 93-09 Study Schema with 5 "options" *Disease evaluation earlier **Disease evaluation later However, the objectives of the trial make it clear that this trial is designed to compare 2 treatment strategies, chemotherapy and radiation with and without surgery, so this study is better modeled as a 2-arm trial, but with major "skip forward" arrows for some subjects, as illustrated in the following diagram. This diagram also shows more detail within the Induction Chemo + RT and Additional Chemo blocks than the preceding diagram. Both the induction and additional chemotherapy are given in 2 cycles. The second induction cycle is different for the 2 arms, since radiation therapy for those assigned to the non-surgery arm includes a “boost” which those assigned to the surgery arm do not receive. The next diagram shows the prospective view of this trial. The protocol conceives of treatment as being divided into
2 parts, induction and continuation, so these have been treated as 2 different epochs. This is also an important point
in the trial operationally, the point when subjects are “registered” a second time, and when subjects who will skip forward are identified (i.e., because of disease progression or ineligibility for surgery). Example Trial 7, RTOG-93-09 Prospective View *Disease evaluation earlier **Disease evaluation later The next diagram shows the retrospective view of this trial. The fact that the elements in the study cell for the CR arm in the continuation treatment epoch do not fill the space in the diagram is an artifact of the diagram conventions. Those subjects who do receive surgery will in fact spend a longer time completing treatment and moving into follow-up. Although it is tempting to think of the horizontal axis of these diagrams as a timeline, this can sometimes


## Page 401

be misleading. The diagrams are not necessarily to scale in the sense that the length of the block representing an element represents its duration, and elements that line up on the same vertical line in the diagram may not occur at the same relative time within the study. Example Trial 7, RTOG 93-09 Retrospective View *Disease evaluation earlier **Disease evaluation later The following table shows the trial design matrix for this 2-arm example trial. Trial Design Matrix

The TA dataset reflects that this is a 2-arm trial. ta.xpt

|  | Screen | Induction |  | Continuation |  |  |  |  |  | Follow-up |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CR | Screen | Initial Chemo +<br>RT | Chemo + RT (non-<br>Surgery) | Chemo |  |  | Chemo |  |  | Off Treatment Follow-<br>up |
| CRS | Screen | Initial Chemo +<br>RT | Chemo + RT (Surgery) | 3-5 w<br>Rest | Surgery | 4-6 w<br>Rest |  | Chemo | Chemo | Off Treatment<br>Follow-up |

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX7 | TA | 1 | CR | 1 | SCRN | Screen | Randomized<br>to CR |  | SCREENING |
| 2 | EX7 | TA | 1 | CR | 2 | ICR | Initial Chemo<br>+ RT |  |  | INDUCTION<br>TREATMENT |
| 3 | EX7 | TA | 1 | CR | 3 | CRNS | Chemo+RT<br>(non-<br>Surgery) |  | If progression, skip to<br>Follow-up. | INDUCTION<br>TREATMENT |
| 4 | EX7 | TA | 1 | CR | 4 | C | Chemo |  |  | CONTINUATION<br>TREATMENT |
| 5 | EX7 | TA | 1 | CR | 5 | C | Chemo |  |  | CONTINUATION<br>TREATMENT |
| 6 | EX7 | TA | 1 | CR | 6 | FU | Off<br>Treatment<br>Follow-up |  |  | FOLLOW-UP |
| 7 | EX7 | TA | 2 | CRS | 1 | SCRN | Screen | Randomized<br>to CRS |  | SCREENING |
| 8 | EX7 | TA | 2 | CRS | 2 | ICR | Initial Chemo<br>+ RT |  |  | INDUCTION<br>TREATMENT |
| 9 | EX7 | TA | 2 | CRS | 3 | CRS | Chemo+RT<br>(Surgery) |  | If progression, skip to<br>Follow-up. If no<br>progression, but subject is<br>ineligible for or does not<br>consent to surgery, skip to<br>Chemo. | INDUCTION<br>TREATMENT |
| 10 | EX7 | TA | 2 | CRS | 4 | R3 | 3-5 week rest |  |  | CONTINUATION<br>TREATMENT |
| 11 | EX7 | TA | 2 | CRS | 5 | SURG | Surgery |  |  | CONTINUATION<br>TREATMENT |


## Page 402

7.2.1.1 Trial Arms Issues
Distinguishing Between Branches and Transitions Both the Branch and Transition columns contain rules, but the 2 columns represent 2 different types of rules. Branch rules represent forks in the trial flowchart, giving rise to separate arms. The rule underlying a branch in the trial design appears in multiple records, once for each "fork" of the branch. Within any one record, there is no choice (no "if" clause) in the value of the branch condition. For example, the value of TABRANCH for a record in arm A is "Randomized to Arm A" because a subject in arm A must have been randomized to arm A. Transition rules are used for choices within an arm. The value for TATRANS does contain a choice (an "if" clause). In Example Trial 4, subjects who receive 1, 2, 3, or 4 cycles of treatment A are all considered to belong to arm A. In modeling a trial, decisions may have to be made about whether a decision point in the flow chart represents the separation of paths that represent different arms, or paths that represent variations within the same arm, as illustrated in the discussion of Example Trial 7. This decision will depend on the comparisons of interest in the trial. Some trials refer to groups of subjects who follow a particular path through the trial as "cohorts," particularly if the groups are formed successively over time. The term "cohort" is used with different meanings in different protocols and does not always correspond to an arm. Subjects Not Assigned to an Arm Some trial subjects may drop out of the study before they reach all of the branch points in the trial design. In the Demographics (DM) domain, the values of ARM and ARMCD must be supplied for such subjects, but the special values used for these subjects should not be included in the Trial Arms (TA) dataset; only complete arm paths should be described in the TA dataset. In Section 5.2, Demographics, assumption 4 describes special ARM and ARMCD values used for subjects who do not reach the first branch point in a trial. When a trial design includes 2 or more branches, special values of ARM and ARMCD may be needed for subjects who pass through the first branch point, but drop out before the final branch point. See DM Example 3 for how to represent ARM and ARMCD values for such trials. Defining Epochs The series of examples for the TA dataset provides a variety of scenarios and guidance about how to assign epoch in those scenarios. In general, assigning epochs for blinded trials is easier than for unblinded trials. The blinded view of the trial will generally make the possible choices clear. For unblinded trials, the comparisons that will be made between arms can guide the definition of epochs. For trials that include many variant paths within an arm, comparisons of arms will mean that subjects on a variety of paths will be included in the comparison, and this is likely to lead to definition of broader epochs. Rule Variables The Branch and Transition columns shown in the example tables are variables with a Role of “Rule.” The values of a Rule variable describe conditions under which something is planned to happen. At the moment, values of Rule variables are text. At some point in the future, it is expected that a mechanism to provide machine-readable rules will become available. Other Rule variables are present in the Trial Elements (TE) and Trial Visits (TV) datasets.
7.2.2 Trial Elements (TE)
TE – Description/Overview A trial design domain that contains the element code that is unique for each element, the element description, and the rules for starting and ending an element.

| Row | STUDYID | DOMAIN | ARMCD | ARM | TAETORD | ETCD | ELEMENT | TABRANCH | TATRANS | EPOCH |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 12 | EX7 | TA | 2 | CRS | 6 | R4 | 4-6 week rest |  |  | CONTINUATION<br>TREATMENT |
| 13 | EX7 | TA | 2 | CRS | 7 | C | Chemo |  |  | CONTINUATION<br>TREATMENT |
| 14 | EX7 | TA | 2 | CRS | 8 | C | Chemo |  |  | CONTINUATION<br>TREATMENT |
| 15 | EX7 | TA | 2 | CRS | 9 | FU | Off<br>Treatment<br>Follow-up |  |  | FOLLOW-UP |


## Page 403

The Trial Elements (TE) dataset contains the definitions of the elements that appear in the Trial Arms (TA) dataset. An element may appear multiple times in the TA table because it appears either (1) in multiple arms, (2) multiple times within an arm, or (3) both. However, an element will appear only once in the TE table. Each row in the TE dataset may be thought of as representing a "unique element" in the same sense of "unique" as a CRF template page for a collecting certain type of data referred to as "unique page." For instance, a CRF might be described as containing 87 pages, but only 23 unique pages. By analogy, the trial design matrix in Example Trial
1 (see Section 7.2.1, Trial Arms) has 9 study cells, each of which contains 1 element, but the same trial design
matrix contains only 5 unique elements, so the TE dataset for that trial has only 5 records. An element is a building block for creating study cells, and an arm is composed of study cells. Or, from another point of view, an arm is composed of elements; that is, the trial design assigns subjects to arms, which comprise a sequence of steps called elements. Trial elements represent an interval of time that serves a purpose in the trial and are associated with certain activities affecting the subject. “Week 2 to week 4” is not a valid element. A valid element has a name that describes the purpose of the element and includes a description of the activity or event that marks the subject's transition into the element as well as the conditions for leaving the element. TE – Specification te.xpt, Trial Elements — Trial Design. One record per planned Element, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TE – Assumptions
1. There are no gaps between elements. The instant one element ends, the next element begins. A subject
spends no time “between” elements.
2. The ELEMENT (Description of the Element) variable usually indicates the treatment being administered
during an element, or, if no treatment is being administered, the other activities that are the purpose of this period of time (e.g., "Screening", "Follow-up", "Washout"). In some cases, this time period may be quite passive (e.g., "Rest"; "Wait, for disease episode").
3. The TESTRL (Rule for Start of Element) variable identifies the event that marks the transition into this
element. For elements that involve treatment, this is the start of treatment.
4. For elements that do not involve treatment, TESTRL can be more difficult to define. For washout and
follow-up elements, which always follow treatment elements, the start of the element may be defined relative to the end of a preceding treatment. For example, a washout period might be defined as starting 24 or 48 hours after the last dose of drug for the preceding treatment element or epoch. This definition is not totally independent of the TA dataset, because it relies on knowing where in the trial design the element is used, and that it always follows a treatment element. Defining a clear starting point for the start of a nontreatment element that always follows another non-treatment element can be particularly difficult. The

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms, Codelist<br>or Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | TE | Identifier | Two-character abbreviation for the domain. | Req |
| ETCD | Element Code | Char | * | Topic | ETCD (the companion to ELEMENT) is limited to 8 | Req |
|  |  |  |  |  | characters and does not have special character restrictions. |  |
|  |  |  |  |  | These values should be short for ease of use in |  |
|  |  |  |  |  | programming, but it is not expected that ETCD will need to |  |
|  |  |  |  |  | serve as a variable name. |  |
| ELEMENT | Description of<br>Element | Char | * | Synonym<br>Qualifier | The name of the element. | Req |
| TESTRL | Rule for Start of<br>Element | Char |  | Rule | Describes condition for beginning element. | Req |
| TEENRL | Rule for End of<br>Element | Char |  | Rule | Describes condition for ending element. Either TEENRL or | Perm |
|  |  |  |  |  | TEDUR must be present for each element. |  |
| TEDUR | Planned<br>Duration of<br>Element | Char | ISO 8601<br>duration | Timing | Planned duration of element in ISO 8601 format. Used when | Perm |
|  |  |  |  |  | the rule for ending the element is applied after a fixed |  |
|  |  |  |  |  | duration. |  |


## Page 404

transition may be defined by a decision-making activity such as enrollment or randomization. For example, every arm of a trial that involves treating disease episodes might start with a screening element followed by an element that consists of waiting until a disease episode occurs. The activity that marks the beginning of the wait element might be randomization.
5. TESTRL for a treatment element may be thought of as “active” whereas the start rule for a non-treatment
element—particularly a follow-up or washout element—may be “passive.” The start of a treatment element will not occur until a dose is given, no matter how long that dose is delayed. Once the last dose is given, the start of a subsequent non-treatment element is inevitable, as long as another dose is not given.
6. Note that the date/time of the event described in TESTRL will be used to populate the date/times in the
Subject Elements (SE) dataset, so the date/time of the event should be captured in the CRF.
7. Specifying TESTRL for an element that serves the first element of an arm in the TA dataset involves
defining the start of the trial. In the examples in this document, obtaining informed consent has been used as "Trial Entry."
8. TESTRL should be expressed without referring to arm. If the element appears in more than 1 arm in the TA
dataset, then the element description (ELEMENT) must not refer to any arms.
9. TESTRL should be expressed without referring to epoch. If the element appears in more than 1 epoch in
the TA dataset, then the Element description (ELEMENT) must not refer to any epochs.
10. For a blinded trial, it is useful to describe TESTRL in terms that separate the properties of the event that are
visible to blinded participants from the properties that are visible only to those who are unblinded. For treatment elements in blinded trials, wording such as the following is suitable: "First dose of study drug for a treatment epoch, where study drug is X."
11. Element end rules are rather different from element start rules. The actual end of one element is the
beginning of the next element. Thus, the element end rule does not give the conditions under which an element does end, but the conditions under which it should end or is planned to end.
12. At least 1 of TEENRL and TEDUR must be populated. Both may be populated.
13. TEENRL describes the circumstances under which a subject should leave this element. Element end rules
may depend on a variety of conditions. For instance, a typical criterion for ending a rest element between oncology chemotherapy-treatment element would be, “15 days after start of element and after WBC values have recovered." The TA dataset, not the TE dataset, describes where the subject moves next, so TEENRL must be expressed without referring to arm.
14. TEDUR serves the same purpose as TEENRL for the special (but very common) case of an element with a
fixed duration. TEDUR is expressed in ISO 8601. For example, a TEDUR value of P6W is equivalent to a TEENRL of "6 weeks after the start of the element."
15. Note that elements that have different start and end rules are different elements and must have different
values of ELEMENT and ETCD. For instance, elements that involve the same treatment but have different durations are different elements. The same applies to non-treatment elements. For instance, a washout with a fixed duration of 14 days is different from a washout that is to end after 7 days if drug cannot be detected in a blood sample, or after 14 days otherwise. TE – Examples Both of the trials in TA Examples 1 and 2 (see Section 7.2.1, Trial Arms) are assumed to have fixed-duration elements. The wording in TESTRL is intended to separate the description of the event that starts the element into the part that would be visible to a blinded participant in the trial (e.g., "First dose of a treatment epoch") from the part that is revealed when the study is unblinded (e.g., "where dose is 5 mg"). Care must be taken in choosing these descriptions to be sure that they are arm- and epoch-neutral. For instance, in a crossover trial such as TA Example Trial 3, where an element may appear in 1 of multiple epochs, the wording must be appropriate for all possible epochs (e.g., "OPEN LABEL TREATMENT"). The SDS Team is considering adding a separate variable to the TE dataset that would hold information on the treatment that is associated with an element. This would make it clearer which elements are "treatment elements” and, therefore, which epochs contain treatment elements and thus are "treatment Epochs."


## Page 405

Example 1 This example shows the TE dataset for TA Example Trial 1. te.xpt

Example 2 This example shows the TE dataset for TA Example Trial 2. te.xpt

| Row | STUDYID | DOMAIN | ETCD | ELEMENT | TESTRL T | EENRL | TEDUR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX1 | TE | SCRN | Screen | Informed consent 1 | week after start of Element | P7D |
| 2 | EX1 | TE | RI | Run-In | Eligibility confirmed 2 | weeks after start of Element | P14D |
| 3 | EX1 | TE | P | Placebo | First dose of study drug, where drug is placebo 2 | weeks after start of Element | P14D |
| 4 | EX1 | TE | A | Drug A | First dose of study drug, where drug is Drug A 2 | weeks after start of Element | P14D |
| 5 | EX1 | TE | B | Drug B | First dose of study drug, where drug is Drug B 2 | weeks after start of Element | P14D |

Example 3 The TE dataset for TA Example Trial 4 illustrates element end rules for elements that are not all of fixed duration. The screen element in this study can be up to 2 weeks long, but because it may end earlier it is not of fixed duration. The rest element has a variable length, depending on how quickly WBC recovers. Note that the start rules for the A and B elements have been written to be suitable for a blinded study. te.xpt

| Row | STUDYID | DOMAIN | ETCD | ELEMENT | TESTRL | TEENRL | TEDUR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX2 | TE | SCRN | Screen | Informed consent | 2 weeks after start of<br>Element | P14D |
| 2 | EX2 | TE | P | Placebo | First dose of a treatment Epoch, where dose is placebo | 2 weeks after start of<br>Element | P14D |
| 3 | EX2 | TE | 5 | 5 mg | First dose of a treatment Epoch, where dose is 5 mg drug | 2 weeks after start of<br>Element | P14D |
| 4 | EX2 | TE | 10 | 10 mg | First dose of a treatment Epoch, where dose is 10 mg<br>drug | 2 weeks after start of<br>Element | P14D |
| 5 | EX2 | TE | REST | Rest | 48 hrs after last dose of preceding treatment Epoch | 1 week after start of Element | P7D |
| 6 | EX2 | TE | FU | Follow-up | 48 hrs after last dose of third treatment Epoch | 3 weeks after start of<br>Element | P21D |

7.2.2.1 Trial Elements Issues
Granularity of Trial Elements Deciding how finely to divide trial time when identifying trial elements is a matter of judgment, as illustrated by the following examples:
1. TA Example Trial 2 was represented using 3 treatment epochs separated by 2 washout epochs and followed
by a follow-up epoch. This might have been modeled using 3 treatment epochs that included both the 2week treatment period and the 1-week rest period. Because the first week after the third treatment period would be included in the third treatment epoch, the follow-up epoch would then have a duration of 2 weeks.
2. In TA Example Trials 4, 5, and 6, separate treatment and rest elements were identified. However, the
combination of treatment and rest could be represented as a single element.
3. A trial might include a dose titration, with subjects receiving increasing doses on a weekly basis until
certain conditions are met. The trial design could be modeled in any of the following ways: a. Using several 1-week elements at specific doses, followed by an element of variable length at the chosen dose

| Row | STUDYID | DOMAIN | ETCD | ELEMENT | TESTRL | TEENRL | TEDUR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | EX4 | TE | SCRN | Screen | Informed Consent | Screening assessments are complete, up to 2<br>weeks after start of Element |  |
| 2 | EX4 | TE | A | Trt A | First dose of treatment Element,<br>where drug is Treatment A | 5 days after start of Element | P5D |
| 3 | EX4 | TE | B | Trt B | First dose of treatment Element,<br>where drug is Treatment B | 5 days after start of Element | P5D |
| 4 | EX4 | TE | REST | Rest | Last dose of previous treatment cycle<br>+ 24 hrs | At least 16 days after start of Element and WBC<br>recovered |  |
| 5 | EX4 | TE | FU | Follow-up | Decision not to treat further | 4 weeks | P28D |


## Page 406

b. As a titration element of variable length followed by a constant dosing element of variable length c. One element with dosing determined by titration. The choice of elements used to represent this dose titration will depend on the objectives of the trial and how the data will be analyzed and reported. If it is important to examine side effects or lab values at each individual dose, the first model is appropriate. If it is important only to identify the time to completion of titration, the second model might be appropriate. If the titration process is routine and is of little interest, the third model might be adequate for the purposes of the trial. Distinguishing Elements, Study Cells, and Epochs It is easy to confuse elements, which are reusable trial building blocks, with study cells (which contain the elements for a particular epoch and Arm) and with epochs (which are time periods for the trial as a whole). In part, this is because many trials have epochs for which the same element appears in all arms. In other words, in the trial design matrix for many trials, there are columns (Epochs) in which all the study cells have the same contents. It also is natural to use the same name (e.g., screen, follow-up) for both such an epoch and the single element that appears within it. Confusion can also arise from the fact that in the blinded treatment portions of blinded trials, blinded participants do not know which element a subject is in, but do know what epoch the subject is in. In describing a trial, one way to avoid confusion between elements and epochs is to include "Element" or "Epoch" in the values of ELEMENT or EPOCH when these values (e.g., screening, follow-up) would otherwise be the same. It becomes tedious to do this in every case, but can be useful to resolve confusion when it arises or is likely to arise. The difference between epoch and element is perhaps clearest in crossover trials. In TA Example Trial 2, as for most crossover trials, the analysis of pharmacokinetic (PK) results would include both treatment and period effects in the model. “Treatment effect” derives from element (placebo, 5 mg, 10 mg), whereas “period effect” derives from the epoch (first, second, or third treatment epoch). Transitions Between Elements The transition between one element and the next can be thought of as a 3-step process:

Note that the subject is not "in limbo" during this process. The subject remains in the current element until step 3, at which point the subject transitions to the new element. There are no gaps between elements. As illustrated in the table, executing a transition depends on information that is split between the TE and the TA datasets. It can be useful, in the process of working out the Trial Design (TD) datasets, to create a dataset that supplements the TA dataset with the TESTRL, TEENRL, and TEDUR variables, so that full information on the transitions is easily accessible. However, such a working dataset is not an SDTM dataset, and should not be submitted. The following table shows a fragment of such a table for TA Example Trial 4. Note that
• for all records that contain a particular element, all the TE variable values are exactly the same; and
• when both TABRANCH and TATRANS are blank, the implicit decision in step 2 is that the subject moves
to the next element in sequence for the arm.

| Step | Step question | How step question is answered by information in the TA datasets |
| --- | --- | --- |
| 1 | Should the subject leave the current<br>element? | The criteria for ending the current element are in TEENRL in the TE dataset. |
| 2 | Which element should the subject enter<br>next? | If there is a branch point at this point in the trial, evaluate criteria described in<br>TABRANCH (e.g., randomization results) in the TA dataset. Otherwise, if<br>TATRANS in the TA dataset is populated in this arm at this point, follow those<br>instructions. Otherwise, move to the next element in this arm as specified by<br>TAETORD in the TA dataset. |
| 3 | What does the subject do to enter the<br>next element? | The action or event that marks the start of the next element is specified in<br>TESTRL in the TE dataset. |


## Page 407

special.xpt

Note that rows 2 and 4 of this dataset involve the same element (Trt A); thus, TESTRL is the same for both. The activity that marks a subject's entry into the fourth element in arm A is "First dose of treatment Element, where drug is Treatment A." This is not the subject's very first dose of treatment A, but it is their first dose in this element.
7.3 Schedule for Assessments (TV, TD, and TM)
This section contains the Trial Design (TD) datasets that describe:
• The protocol-defined planned schedule of subject encounters at the healthcare facility where the study is
being conducted (Section 7.3.1, Trial Visits (TV))
• The planned schedule of efficacy assessments related to the disease under study (Section 7.3.2, Trial
Disease Assessments (TD))
• The things (events, interventions, or findings) which, if and when they happen, are the occasion for
assessments planned in the protocol (Section 7.3.3, Trial Disease Milestones (TM)) The Trial Visits (TV) and TD datasets provide the planned scheduling of assessments to which a subject’s actual visits and disease assessments can be compared.
7.3.1 Trial Visits (TV)
TV – Description/Overview A trial design domain that contains the planned order and number of visits in the study within each arm. Visits are defined as "clinical encounters" and are described using the timing variables VISIT, VISITNUM, and VISITDY. Protocols define visits in order to describe assessments and procedures that are to be performed at the visits. TV – Specification tv.xpt, Trial Visits — Trial Design. One record per planned Visit per Arm, Tabulation.

| Row | ARM | EPOCH | TAETORD | ELEMENT | TESTRL | TEENRL | TEDUR | TABRANCH | TATRANS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | A | Screen | 1 | Screen | Informed Consent | Screening assessments<br>are complete, up to 2<br>weeks after start of<br>Element |  | Randomized<br>to A |  |
| 2 | A | Treatment | 2 | Trt A | First dose of<br>treatment in Element,<br>where drug is<br>Treatment A | 5 days after start of<br>Element | P5D |  |  |
| 3 | A | Treatment | 3 | Rest | Last dose of previous<br>treatment cycle + 24<br>hrs | 16 days after start of<br>Element and WBC<br>recovers |  |  | If disease<br>progression, go to<br>Follow-up Epoch |
| 4 | A | Treatment | 4 | Trt A | First dose of<br>treatment in Element,<br>where drug is<br>Treatment A | 5 days after start of<br>Element | P5D |  |  |

| Variable<br>Name | Variable<br>Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study<br>Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | TV | Identifier | Two-character abbreviation for the domain. | Req |
| VISITNUM | Visit Number | Num |  | Topic | Clinical encounter number. Numeric version of VISIT, used for | Req |
|  |  |  |  |  | sorting. |  |
| VISIT | Visit Name | Char |  | Synonym<br>Qualifier | Description of clinical encounter. This is often defined in the | Req |
|  |  |  |  |  | protocol. Used in addition to VISITNUM and/or VISITDY as a text |  |
|  |  |  |  |  | description of the clinical encounter. |  |
| VISITDY | Planned<br>Study Day of<br>Visit | Num |  | Timing | Planned study day of VISIT. Due to its sequential nature, used for | Perm |
|  |  |  |  |  | sorting. |  |


## Page 408

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TV – Assumptions
1. Although the general structure of the Trial Visits (TV) dataset is "One Record per Planned Visit per Arm,"
for many clinical trials—particularly blinded clinical trials—the schedule of visits is the same for all arms, and the structure of the TV dataset will be "One Record per Planned Visit." If the schedule of visits is the same for all arms, ARMCD should be left blank for all records in the TV dataset. For trials with trial visits that are different for different arms (e.g., Example Trial 7 in Section 7.2.1, Trial Arms), ARMCD and ARM should be populated for all records. If some visits are the same for all arms, and some visits differ by arm, then ARMCD and ARM should be populated for all records, to ensure clarity, even though this will mean creating near-duplicate records for visits that are the same for all arms.
2. A visit may start in one element and end in another. This means that a visit may start in one epoch and end
in another. For example, if one of the activities planned for a visit is the administration of the first dose of study drug, the visit might start in the screen epoch and end in a treatment epoch.
3. TVSTRL describes the scheduling of the visit and should reflect the wording in the protocol. In many trials,
all visits are scheduled relative to the study's day 1 (RFSTDTC). In such trials, it is useful to include VISITDY, which is, in effect, a special case representation of TVSTRL.
4. Note that there is a subtle difference between the following 2 examples. In the first case, if visit 3 were
delayed for some reason, visit 4 would be unaffected. In the second case, a delay to visit 3 would result in visit 4 being delayed as well. a. Case 1: Visit 3 starts 2 weeks after RFSTDTC. Visit 4 starts 4 weeks after RFSTDTC. b. Case 2: Visit 3 starts 2 weeks after RFSTDTC. Visit 4 starts 2 weeks after visit 3.
5. Many protocols do not give any information about visit ends because visits are assumed to end on the same
day they start. In such a case, TVENRL may be left blank to indicate that the visit ends on the same day it starts. Care should be taken to assure that this is appropriate; common practice may be to record data collected over more than 1 day as occurring within a single visit. Screening visits may be particularly prone to collection of data over multiple days. The examples for this domain show how TVENRL could be populated.
6. The values of VISITNUM in the TV dataset are the valid values of VISITNUM for planned visits. Any
values of VISITNUM that appear in subject-level datasets that are not in the TV dataset are assumed to correspond to unplanned visits. This applies, in particular, to the subject-level dataset; see Section 5.5, Subject Visits, for additional information about handling unplanned visits. If a subject-level dataset includes both VISITNUM and VISIT, then records that include values of VISITNUM that appear in the TV dataset should also include the corresponding values of VISIT from the TV dataset.

| Variable<br>Name | Variable<br>Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| ARMCD | Planned Arm<br>Code | Char | * | Record<br>Qualifier | 1. ARMCD is limited to 20 characters and does not have special | Exp |
|  |  |  |  |  | character restrictions. The maximum length of ARMCD is |  |
|  |  |  |  |  | longer than for other "short" variables to accommodate the kind |  |
|  |  |  |  |  | of values that are likely to be needed for crossover trials. For |  |
|  |  |  |  |  | example, if ARMCD values for a 7-period crossover were |  |
|  |  |  |  |  | constructed using 2-character abbreviations for each treatment |  |
|  |  |  |  |  | and separating hyphens, the length of ARMCD values would be |  |
|  |  |  |  |  | 20. |  |
|  |  |  |  |  | 2. If the timing of visits for a trial does not depend on which arm a |  |
|  |  |  |  |  | subject is in, then ARMCD should be null. |  |
| ARM | Description of<br>Planned Arm | Char | * | Synonym<br>Qualifier | 1. Name given to an arm or treatment group. | Perm |
|  |  |  |  |  | 2. If the timing of visits for a trial does not depend on which arm a |  |
|  |  |  |  |  | subject is in, then Arm should be left blank. |  |
| TVSTRL | Visit Start<br>Rule | Char |  | Rule | Rule describing when the visit starts, in relation to the sequence of | Req |
|  |  |  |  |  | elements. |  |
| TVENRL | Visit End Rule | Char |  | Rule | Rule describing when the visit ends, in relation to the sequence of | Perm |
|  |  |  |  |  | elements. |  |


## Page 409

TV – Examples Example 1 The following diagram represents visits as numbered "flags" with visit numbers. Each flag has 2 supports, one at the beginning of the visit and the other at the end of the visit. Note that visits 2 and 3 span epoch transitions. In other words, the transition event that marks the beginning of the run-in epoch (confirmation of eligibility) occurs during visit 2, and the transition event that marks the beginning of the treatment epoch (the first dose of study drug) occurs during visit 3. Example Trial 1, Parallel Design Planned Visits Two TV datasets are shown for this trial. The first shows a somewhat idealized situation, where the protocol has provided specific timings for the visits. The second shows a more common situation, where the timings have been described only loosely. tv.xpt

tv.xpt

| Row | STUDYID | DOMAIN | VISITNUM | TVSTRL | TVENRL |
| --- | --- | --- | --- | --- | --- |
| 1 | EX1 | TV | 1 | Start of Screen Epoch | 1 hour after start of Visit |
| 2 | EX1 | TV | 2 | 30 minutes before end of Screen Epoch | 30 minutes after start of Run-in Epoch |
| 3 | EX1 | TV | 3 | 30 minutes before end of Run-in Epoch | 1 hour after start of Treatment Epoch |
| 4 | EX1 | TV | 4 | 1 week after start of Treatment Epoch | 1 hour after start of Visit |
| 5 | EX1 | TV | 5 | 2 weeks after start of Treatment Epoch | 1 hour after start of Visit |

Although the start and end rules in this example reference the starts and ends of epochs, the start and end rules of some visits for trials with epochs that span multiple elements will need to reference elements rather than epochs. When an arm includes repetitions of the same element, it may be necessary to use TAETORD as well as an element name to specify when a visit is to occur.
7.3.1.1 Trial Visits Issues
Identifying Trial Visits In general, a trial's visits are defined in its protocol. The term “visit” reflects the fact that data in outpatient studies is usually collected during a physical visit by the subject to a clinic. Sometimes a trial visit defined by the protocol may not correspond to a physical visit. It may span multiple physical visits, as when screening data is collected over several clinic visits but recorded under one TV name (VISIT) and number (VISITNUM). A trial visit also

| Row | STUDYID | DOMAIN | VISITNUM | TVSTRL | TVENRL |
| --- | --- | --- | --- | --- | --- |
| 1 | EX1 | TV | 1 | Start of Screen Epoch |  |
| 2 | EX1 | TV | 2 | On the same day as, but before, the end of the<br>Screen Epoch | On the same day as, but after, the start of the Run-in<br>Epoch |
| 3 | EX1 | TV | 3 | On the same day as, but before, the end of the<br>Run-in Epoch | On the same day as, but after, the start of the<br>Treatment Epoch |
| 4 | EX1 | TV | 4 | 1 week after start of Treatment Epoch |  |
| 5 | EX1 | TV | 5 | 2 weeks after start of Treatment Epoch | At Trial Exit |


## Page 410

may represent only a portion of an extended physical visit, as when a trial of in-patients collects data under multiple trial visits for a single hospital admission. Diary data and other data collected outside a clinic may not fit the usual concept of a trial visit, but the planned times of collection of such data may be described as “visits” in the TV dataset if desired. Trial Visit Rules Visit start rules are different from element start rules in that they usually describe when a visit should occur; element start rules describe the moment at which an element is considered to start. There are usually gaps between visits, periods of time that do not belong to any visit, so it is usually not necessary to identify the moment when one visit stops and another starts. However, some trials of hospitalized subjects may divide time into visits in a manner more like that used for elements, and a transition event may need to be defined in such cases. Visit start rules are usually expressed relative to the start or end of an element or epoch (e.g., "1-2 hours before end of First Wash-out", "8 weeks after end of 2nd Treatment Epoch"). Note that the visit may or may not occur during the element used as the reference for the visit start rule. For example, a trial with elements based on treatment of disease episodes might plan a visit 6 months after the start of the first treatment period, regardless of how many disease episodes have occurred. Visit end rules are similar to element end rules, describing when a visit should end. They may be expressed relative to the start or end of an element or epoch, or relative to the start of the visit. The timings of visits relative to elements may be expressed in terms that cannot be easily quantified. For instance, a protocol might instruct that at a baseline visit the subject be randomized, given the study drug, and instructed to take the first dose of study drug X at bedtime that night. This baseline visit is thus started and ended before the start of the treatment epoch, but we don't know how long before the start of the treatment epoch the visit will occur. The trial start rule might contain the value "On the day of, but before, the start of the Treatment Epoch". Visit Schedules Expressed with Ranges Ranges may be used to describe the planned timing of visits (e.g., 12-16 days after the start of 2nd Element), but this is different from the “windows” that may be used in selecting data points to be included in an analysis associated with that visit. For example, although visit 2 was planned for 12-16 days after the start of treatment, data collected 10-18 days after the start of treatment might be included in a visit 1 analysis. The 2 ranges serve different purposes. Contingent Visits Some data collection is contingent on the occurrence of a "trigger" event or disease milestone (see Section 7.3.3, Trial Disease Milestones (TM)). When such planned data collection involves an additional clinic visit, a "contingent" visit may be included in the TV table, with a rule that describes the circumstances under which it will take place. Because values of VISITNUM must be assigned to all records in the TV dataset, a contingent visit included in the TV dataset must have a VISITNUM, but the VISITNUM value might not be a "chronological" value, due to the uncertain timing of a contingent visit. If contingent visits are not included in the TV dataset, then they would be treated as unplanned visits in the Subject Visits (SV) domain (see Section 6.2.8, Subject Visits).
7.3.2 Trial Disease Assessments (TD)
TD – Description/Overview A trial design domain that provides information on the protocol-specified disease assessment schedule, to be used for comparison with the actual occurrence of the efficacy assessments in order to determine whether there was good compliance with the schedule. TD – Specification td.xpt, Trial Disease Assessments — Trial Design. One record per planned constant assessment period, Tabulation.

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |


## Page 411

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TD – Assumptions
1. The purpose of the Trial Disease Assessments (TD) domain is to provide information on planned
scheduling of disease assessments when the scheduling of disease assessments is not necessarily tied to the scheduling of visits. In oncology studies, good compliance with the disease-assessment schedule is essential to reduce the risk of "assessment time bias." The TD domain makes possible an evaluation of assessment time bias from the SDTM, in particular for studies with progression-free survival (PFS) endpoints. TD has limited utility within oncology and was developed specifically with RECIST in mind and where an assessment-time bias analysis is appropriate. It is understood that extending this approach to Cheson and other criteria may not be appropriate or may pose difficulties. It is also understood that this approach may not be necessary in non-oncology studies, although it is available for use if appropriate.
2. A planned schedule of assessments will have a defined start point; the TDANCVAR variable is used to
identify the variable in the ADaM subject-level dataset (ADSL) that holds the “anchor” date. By default, the anchor variable for the first pattern is ANCH1DT. An anchor date must be provided for each pattern of assessments, and each anchor variable must exist in ADSL. TDANCVAR is therefore a Required variable. Anchor date variable names should adhere to ADaM variable naming conventions (e.g. ANCH1DT, ANCH2DT). One anchor date may be used to anchor more than 1 pattern of disease assessments. When that is the case, the appropriate offset for the start of a subsequent pattern, represented as an ISO 8601 duration value, should be provided in the TDSTOFF variable.
3. The TDSTOFF variable is used in conjunction with the anchor date value (from the anchor date variable
identified in TDANCVAR). If the pattern of disease assessments does not start exactly on a date collected on the CRF, this variable will represent the offset between the anchor date value and the start date of the pattern of disease assessments. This may be a positive or zero interval value represented in an ISO 8601 format.

| DOMAIN | Domain<br>Abbreviation | Char | TD | Identifier | Two-character abbreviation for the domain. | Req |
| --- | --- | --- | --- | --- | --- | --- |
| TDORDER | Sequence of<br>Planned<br>Assessment<br>Schedule | Num |  | Timing | A number given to ensure ordinal sequencing of the planned | Req |
|  |  |  |  |  | assessment schedules within a trial. |  |
| TDANCVAR | Anchor Variable<br>Name | Char |  | Timing | A reference to the date variable name that provides the start | Req |
|  |  |  |  |  | point from which the planned disease assessment schedule is |  |
|  |  |  |  |  | measured. This must be a referenced from the ADaM ADSL |  |
|  |  |  |  |  | dataset (e.g., "ANCH1DT"). Note: TDANCVAR will contain the |  |
|  |  |  |  |  | name of a reference date variable. |  |
| TDSTOFF | Offset from the<br>Anchor | Char | ISO 8601<br>duration | Timing | A fixed offset from the date provided by the variable referenced | Req |
|  |  |  |  |  | in TDANCVAR. This is used when the timing of planned cycles |  |
|  |  |  |  |  | does not start on the exact day referenced in the variable |  |
|  |  |  |  |  | indicated in TDANCVAR. The value of this variable will be |  |
|  |  |  |  |  | either zero or a positive value and will be represented in ISO |  |
|  |  |  |  |  | 8601 character format. |  |
| TDTGTPAI | Planned<br>Assessment<br>Interval | Char | ISO 8601<br>duration | Timing | The planned interval between disease assessments | Req |
|  |  |  |  |  | represented in ISO 8601 character format. |  |
| TDMINPAI | Planned<br>Assessment<br>Interval<br>Minimum | Char | ISO 8601<br>duration | Timing | The lower limit of the allowed range for the planned interval | Req |
|  |  |  |  |  | between disease assessments represented in ISO 8601 |  |
|  |  |  |  |  | character format. |  |
| TDMAXPAI | Planned<br>Assessment<br>Interval<br>Maximum | Char | ISO 8601<br>duration | Timing | The upper limit of the allowed range for the planned interval | Req |
|  |  |  |  |  | between disease assessments represented in ISO 8601 |  |
|  |  |  |  |  | character format. |  |
| TDNUMRPT | Maximum<br>Number of<br>Actual<br>Assessments | Num |  | Record<br>Qualifier | This variable must represent the maximum number of actual | Req |
|  |  |  |  |  | assessments for the analysis that this disease assessment |  |
|  |  |  |  |  | schedule describes. In a trial where the maximum number of |  |
|  |  |  |  |  | assessments is not defined explicitly in the protocol (e.g., |  |
|  |  |  |  |  | assessments occur until death), TDNUMRPT should represent |  |
|  |  |  |  |  | the maximum number of disease assessments that support the |  |
|  |  |  |  |  | efficacy analysis encountered by any subject across the trial at |  |
|  |  |  |  |  | that point in time. |  |


## Page 412

4. A pattern of assessments consists of a series of intervals of equal duration, each followed by an assessment.
Thus, the first assessment in a pattern is planned to occur at the anchor date (given by the variable named in TDANCVAR) plus the offset (TDSTOFF) plus the target assessment interval (TDTGTPAI). A baseline evaluation is usually not preceded by an interval, and would therefore not be considered part of an assessment pattern.
5. This domain should not be created when the disease assessment schedule may vary for individual subjects
(e.g., when completion of the first phase of a study is event-driven). TD – Examples Example 1 This example shows a study where the disease assessment schedule changes over the course of the study. In this example, there are 3 distinct disease-assessment schedule patterns. A single anchor date variable (TDANCVAR) provides the anchor date for each pattern. The offset variable (TDSTOFF), used in conjunction with the anchor date variable, provides the start point of each pattern of assessments..
• The first disease-assessment schedule pattern starts at the reference start date (identified in the ADSL
ANCH1DT variable) and repeats every 8 weeks for a total of 6 repeated assessments (i.e., week 8, week 16, week 24, week 32, week 40, week 48). Note that there is an upper and lower limit around the planned disease assessment target where the first assessment (8 weeks) could occur as early as day 53 and as late as week 9. This upper and lower limit (-3 days, +1 week) would be applied to all assessments during that pattern.
• The second disease assessment schedule starts from week 48 and repeats every 12 weeks for a total of 4
repeats (i.e., week 60, week 72, week 84, week 96), with respective upper and lower limits of -1 week and + 1 week.
• The third disease assessment schedule starts from week 96 and repeats every 24 weeks (week 120,
week 144, and so on), with respective upper and lower limits of -1 week and + 1 week, for an indefinite length of time. The preceding schematic shows that, for the third pattern, assessments will occur until disease progression; this therefore leaves the pattern open-ended. However, when data is included in an analysis, the total number of repeats can be identified and the highest number of repeat assessments for any subject in that pattern must be recorded in the TDNUMRPT variable on the final pattern record. td.xpt

| Row | STUDYID | DOMAIN | TDORDER | TDANCVAR | TDSTOFF | TDTGTPAI | TDMINPAI | TDMAXPAI | TDNUMRPT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ABC123 | TD | 1 | ANCH1DT | P0D | P8W | P53D | P9W | 6 |
| 2 | ABC123 | TD | 2 | ANCH1DT | P48W | P12W | P11W | P13W | 4 |
| 3 | ABC123 | TD | 3 | ANCH1DT | P96W | P24W | P23W | P25W | 12 |


## Page 413

Example 2 This example shows a crossover study, where subjects are given the period 1 treatment according to the first diseaseassessment schedule until disease progression, then there is a rest period of 28 days prior to the start of period 2 treatment (i.e., re-baseline for period 2). The subjects are then given the period 2 treatment according to the second disease assessment schedule until disease progression. This example also shows how two different reference/anchor dates can be used.
• The Rest element is not represented as a row in the TD dataset, since no disease assessments occur during
the Rest. Note that although the Rest epoch in this example is not important for TD, it is important that it is represented in other trial design datasets. Row 1: Shows the disease assessment schedule for the first treatment period. The diagram above shows that this schedule repeats until disease progression. After the trial ended, the maximum number of repeats in this schedule was determined to be 6, so that is the value in TDNUMRPT for this schedule. Row 2: Shows the disease assessment schedule for the second period. The pattern starts on the date identified in the ADSL variable ANCH2DT and repeats every 8 weeks with respective upper and lower limits of -1 week and + 1 week. The maximum number of repeats that occurred on this schedule was 4. td.xpt

| Row | STUDYID | DOMAIN | TDORDER | TDANCVAR | TDSTOFF | TDTGTPAI | TDMINPAI | TDMAXPAI | TDNUMRPT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ABC123 | TD | 1 | ANCH1DT | P0D | P8W | P53D | P9W | 6 |
| 2 | ABC123 | TD | 2 | ANCH2DT | P0D | P8W | P53D | P9W | 4 |


## Page 414

Example 3 This example shows a study where subjects are randomized to standard treatment or an experimental treatment. The subjects who are randomized to standard treatment are given the option to receive experimental treatment after they end the standard treatment (e.g., due to disease progression on standard treatment). In the randomized treatment epoch, the disease assessment schedule changes over the course of the study. At the start of the extension treatment epoch, subjects are re-baselined, i.e., an extension baseline disease assessment is performed and the disease assessment schedule is restarted). In this example, there are 3 distinct disease-assessment schedule patterns:
• The first disease-assessment schedule pattern starts at the reference start date (identified in the ADSL
ANCH1DT variable) and repeats every 8 weeks for a total of 6 repeats ( i.e., week 8, week 16, week 24, week 32, week 40, week 48), with respective upper and lower limits of - 3 days and + 1 week.
• The second disease assessment schedule starts from week 48 and repeats every 12 weeks (week 60,
week 72, etc.), with respective upper and lower limits of -1 week and + 1 week, for an indefinite length of time. The preceding schematic shows that, for the second pattern, assessments will occur until disease progression; this therefore leaves the pattern open-ended.
• The third disease assessment schedule starts at the extension reference start date (identified in the ADSL
ANCH2DT variable) from week 96 and repeats every 24 weeks (week 120, week 144, etc.), with respective upper and lower limits of -1 week and + 1 week, for an indefinite length of time. The schematic shows that, for the third pattern, assessments will occur until disease progression; this therefore leaves the pattern openended. For open-ended patterns, the total number of repeats can be identified when the data analysis is performed; the highest number of repeat assessments for any subject in that pattern must be recorded in the TDNUMRPT variable on the final pattern record. td.xpt

| Row | STUDYID | DOMAIN | TDORDER | TDANCVAR | TDSTOFF | TDTGPAI | TDMINPAI | TDMAXPAI | TDNUMRPT |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ABC123 | TD | 1 | ANCH1DT | P0D | P8W | P53D | P9W | 6 |
| 2 | ABC123 | TD | 2 | ANCH1DT | P48W | P12W | P11W | P13W | 17 |
| 3 | ABC123 | TD | 3 | ANCH2DT | P0D | P12W | P11W | P13W | 17 |


## Page 415

7.3.3 Trial Disease Milestones (TM)
TM – Description/Overview A trial design domain that is used to describe disease milestones, which are observations or activities anticipated to occur in the course of the disease under study, and which trigger the collection of data. TM – Specification tm.xpt, Trial Disease Milestones — Trial Design. One record per Disease Milestone type, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TM – Assumptions
1. Disease milestones may be things that would be expected to happen before the study, or things that are
anticipated to happen during the study. The occurrence of disease milestones for particular subjects are represented in the Subject Disease Milestones (SM) dataset.
2. The Trial Disease Milestones (TM) dataset contains a record for each type of disease milestone. The
disease milestone is defined in TMDEF. TM – Examples Example 1 In this diabetes study, initial diagnosis of diabetes and the hypoglycemic events that occur during the trial have been identified as disease milestones of interest. Row 1: Shows that the initial diagnosis is given the MIDSTYPE of "DIAGNOSIS" and is defined in TMDEF. It is not repeating (occurs only once). Row 2: Shows that hypoglycemic events are given the MIDSTYPE of "HYPOGLYCEMIC EVENT", and a definition in TMDEF. (For an actual study, the definition would be expected to include a particular threshold level, rather than the text "threshold level" used in this example.) A subject may experience multiple hypoglycemic events, as indicated by TMRPT = "Y". tm.xpt

| Variable<br>Name | Variable Label | Type | Controlled Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | TM | Identifier | Two-character abbreviation for the domain, which | Req |
|  |  |  |  |  | must be TM. |  |
| MIDSTYPE | Disease Milestone<br>Type | Char |  | Topic | The type of disease milestone. Example: | Req |
|  |  |  |  |  | "HYPOGLYCEMIC EVENT". |  |
| TMDEF | Disease Milestone<br>Definition | Char |  | Variable<br>Qualifier | Definition of the disease milestone. | Req |
| TMRPT | Disease Milestone<br>Repetition Indicator | Char | (NY) | Record<br>Qualifier | Indicates whether this is a disease milestone that can | Req |
|  |  |  |  |  | occur only once ("N") or a type of disease milestone |  |
|  |  |  |  |  | that can occur multiple times ("Y"). |  |

7.4 Trial Eligibility and Summary (TI and TS)
This section contains the Trial Design (TD) datasets that describe:
• Subject eligibility criteria for trial participation (Section 7.4.1, Trial Inclusion/Exclusion Criteria (TI))
• The characteristics of the trial (Section 7.4.2, Trial Summary (TS))
The TI and TS datasets are tabular synopses of parts of the study protocol.

| Row | STUDYID | DOMAIN | MIDSTYPE | TMDEF T | MRPT |
| --- | --- | --- | --- | --- | --- |
| 1 | XYZ | TM | DIAGNOSIS | Initial diagnosis of diabetes, the first time a physician told the subject they had N<br>diabetes |  |
| 2 | XYZ | TM | HYPOGLYCEMIC<br>EVENT | Hypoglycemic Event, the occurrence of a glucose level below (threshold level) Y |  |


## Page 416

7.4.1 Trial Inclusion/Exclusion Criteria (TI)
TI – Proposed Removal of Variable TIRL The variable TIRL was included in the Trial Inclusion/Exclusion Criteria (TI) domain in anticipation of developing a way to represent eligibility criteria in a computer-executable manner. However, such a method has not been developed, and it is not clear that an SDTM dataset would be the best place to represent such a computer-executable representation. TI – Description/Overview A trial design domain that contains one record for each of the inclusion and exclusion criteria for the trial. This domain is not subject oriented. TI contains all the inclusion and exclusion criteria for the trial, and thus provides information that may not be present in the subject-level data on inclusion and exclusion criteria. The IE domain (described in Section 6.3.4, Inclusion/Exclusion Criteria Not Met) contains records only for inclusion and exclusion criteria that subjects did not meet. TI – Specification ti.xpt, Trial Inclusion/Exclusion Criteria — Trial Design. One record per I/E criterion, Tabulation.

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TI – Assumptions
1. If inclusion/exclusion criteria were amended during the trial, then each complete set of criteria must be
included in the TI domain. TIVERS is used to distinguish between the versions.
2. Protocol version numbers should be used to identify criteria versions, although there may be more versions
of the protocol than versions of the inclusion/exclusion criteria. For example, a protocol might have versions 1, 2, 3, and 4, but if the inclusion/exclusion criteria in version 1 were unchanged through versions
2 and 3, and changed only in version 4, then there would be 2 sets of inclusion/exclusion criteria in TI: one
for version 1 and one for version 4.
3. Individual criteria do not have versions. If a criterion changes, it should be treated as a new criterion, with a
new value for IETESTCD. If criteria have been numbered and values of IETESTCD are generally of the form INCL00n or EXCL00n, and new versions of a criterion have not been given new numbers, separate values of IETESTCD might be created by appending letters (e.g., INCL003A, INCL003B).

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms,<br>Codelist or<br>Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | TI | Identifier | Two-character abbreviation for the domain. | Req |
| IETESTCD | Incl/Excl Criterion<br>Short Name | Char | * | Topic | Short name IETEST. It can be used as a column name | Req |
|  |  |  |  |  | when converting a dataset from a vertical to a horizontal |  |
|  |  |  |  |  | format. The value in IETESTCD cannot be longer than 8 |  |
|  |  |  |  |  | characters, nor can it start with a number (e.g., "1TEST" is |  |
|  |  |  |  |  | not valid). IETESTCD cannot contain characters other than |  |
|  |  |  |  |  | letters, numbers, or underscores. The prefix "IE" is used to |  |
|  |  |  |  |  | ensure consistency with the IE domain. |  |
| IETEST | Inclusion/Exclusion<br>Criterion | Char | * | Synonym<br>Qualifier | Full text of the inclusion or exclusion criterion. The prefix | Req |
|  |  |  |  |  | "IE" is used to ensure consistency with the IE domain. |  |
| IECAT | Inclusion/Exclusion<br>Category | Char | (IECAT) | Grouping<br>Qualifier | Used for categorization of the inclusion or exclusion criteria. | Req |
| IESCAT | Inclusion/Exclusion<br>Subcategory | Char | * | Grouping<br>Qualifier | A further categorization of the exception criterion. Can be | Perm |
|  |  |  |  |  | used to distinguish criteria for a sub-study or to categorize |  |
|  |  |  |  |  | as major or minor exceptions. Examples: "MAJOR", |  |
|  |  |  |  |  | "MINOR". |  |
| TIRL | Inclusion/Exclusion<br>Criterion Rule | Char |  | Rule | Rule that expresses the criterion in computer-executable | Perm |
|  |  |  |  |  | form. See Assumption 4. |  |
| TIVERS | Protocol Criteria<br>Versions | Char |  | Record<br>Qualifier | The number of this version of the Inclusion/Exclusion | Perm |
|  |  |  |  |  | criteria. May be omitted if there is only 1 version. |  |


## Page 417

4. IETEST contains the text of the inclusion/exclusion criterion. However, because entry criteria are rules, the
variable TIRL has been included in anticipation of the development of computer-executable rules.
5. If a criterion text is <200 characters, it goes in IETEST; if the text is >200 characters, put meaningful text
in IETEST and describe the full text in the study metadata. See Section 4.5.3.1, Test Name (--TEST) Greater than 40 Characters, for further information. TI – Examples Example 1 This example shows records for a trial that with 2 versions of inclusion/exclusion criteria. Rows 1-3: Show the 2 inclusion criteria and 1 exclusion criterion for version 1 of the protocol. Rows 4-6: Show the inclusion/exclusion criteria for version 2.2 of the protocol, which changed the minimum age for entry from 21 to 18. ti.xpt

7.4.2 Trial Summary (TS)
TS – Description/Overview A trial design domain that contains one record for each trial summary characteristic. This domain is not subject oriented. The Trial Summary (TS) dataset allows the sponsor to submit a summary of the trial in a structured format. Each record in the TS dataset contains the value of a parameter, a characteristic of the trial. For example, TS is used to record basic information about the study such as trial phase, protocol title, and trial objectives. The TS dataset contains information about the planned and actual trial characteristics. TS – Specification ts.xpt, Trial Summary — Trial Design. One record per trial summary parameter value, Tabulation.

| Row | STUDYID | DOMAIN I | ETESTCD | IETEST I | ECAT | TIVERS |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | XYZ | TI I | NCL01 | Has disease under study I | NCLUSION | 1 |
| 2 | XYZ | TI I | NCL02 | Age 21 or greater I | NCLUSION | 1 |
| 3 | XYZ | TI | EXCL01 | Pregnant or lactating | EXCLUSION | 1 |
| 4 | XYZ | TI I | NCL01 | Has disease under study I | NCLUSION | 2.2 |
| 5 | XYZ | TI I | NCL02A | Age 18 or greater I | NCLUSION | 2.2 |
| 6 | XYZ | TI | EXCL01 | Pregnant or lactating | EXCLUSION | 2.2 |

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms, Codelist<br>or Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| STUDYID | Study Identifier | Char |  | Identifier | Unique identifier for a study. | Req |
| DOMAIN | Domain<br>Abbreviation | Char | TS | Identifier | Two-character abbreviation for the domain. | Req |
| TSSEQ | Sequence<br>Number | Num |  | Identifier | Sequence number given to ensure uniqueness within a | Req |
|  |  |  |  |  | parameter. Allows inclusion of multiple records for the |  |
|  |  |  |  |  | same TSPARMCD. |  |
| TSGRPID | Group ID | Char |  | Identifier | Used to tie together a group of related records. | Perm |
| TSPARMCD | Trial Summary<br>Parameter Shor<br>Name | Char<br>t | (TSPARMCD) | Topic | TSPARMCD (the companion to TSPARM) is limited to 8 | Req |
|  |  |  |  |  | characters and does not have special character |  |
|  |  |  |  |  | restrictions. These values should be short for ease of use |  |
|  |  |  |  |  | in programming, but it is not expected that TSPARMCD |  |
|  |  |  |  |  | will need to serve as variable names. Examples: |  |
|  |  |  |  |  | "AGEMIN", "AGEMAX". |  |
| TSPARM | Trial Summary<br>Parameter | Char | (TSPARM) | Synonym<br>Qualifier | Term for the trial summary parameter. The value in | Req |
|  |  |  |  |  | TSPARM cannot be longer than 40 characters. Examples: |  |
|  |  |  |  |  | "Planned Minimum Age of Subjects", "Planned Maximum |  |
|  |  |  |  |  | Age of Subjects". |  |
| TSVAL | Parameter<br>Value | Char | * | Result<br>Qualifier | Value of TSPARM. Example: "ASTHMA" when TSPARM | Exp |
|  |  |  |  |  | value is "Trial Indication". TSVAL can only be null when |  |
|  |  |  |  |  | TSVALNF is populated. Text over 200 characters can be |  |
|  |  |  |  |  | added to additional columns TSVAL1-TSVALn. See |  |
|  |  |  |  |  | Assumption 8. |  |


## Page 418

1In this column, an asterisk (*) indicates that the variable may be subject to controlled terminology. CDISC/NCI codelist values are enclosed in parentheses. TS – Assumptions
1. The intent of this dataset is to provide a summary of trial information. This is not subject-level data.
2. Recipients may specify their requirements for which trial summary parameters should be included under
which circumstances. For example, the US FDA includes such information in their Study Data Technical Conformance Guide.
3. The order of parameters in the examples of TS datasets should not be taken as a requirement. There are no
requirements or expectations about the order of parameters within the TS dataset.
4. The method for treating text >200 characters in TS is similar to that used for the Comments (CO) specialpurpose domain (Section 5.1, Comments). If TSVAL is >200 characters, then it should be split into
multiple variables, TSVAL-TSVALn. See Section 4.5.3.2, Text Strings Greater than 200 Characters in Other Variables.
5. A list of values for TSPARM and TSPARMCD can be found in CDISC Controlled Terminology, available
at https://www.cancer.gov/research/resources/terminology/cdisc.
6. Controlled terminology for TSPARM is extensible. The meaning of any added parameters should be
explained in the metadata for the TS dataset.
7. For a particular trial summary parameter, responses (values in TSVAL) may be numeric, datetimes or
amounts of time represented in ISO8601 format, or text. For some parameters, textual responses may be taken from controlled terminology; for others, responses may be free text.
8. For some trial summary parameters, CDISC Controlled Terminology includes codelists for use with
TSVAL. The associations between trial summary parameters and response codelists are in the TS codetable, available at https://www.cdisc.org/standards/terminology/controlled-terminology. Recipients may also specify controlled terminology for TSVAL. These specifications may be for trial summary parameters for which there is no CDISC Controlled Terminology or they may replace CDISC Controlled Terminology for a trial summary parameter. For example, the US FDA Data Standards Catalog includes terminologies to be used for certain trial summary parameters.
9. There is a code value for TSVALCD only when there is controlled terminology for TSVAL. For example,
when TSPARMCD = "PLANSUB" (Planned Number of Subjects) or TSPARMCD = "TITLE" (Trial Title), then TSVALCD will be null.
10. TSVALNF contains a “null flavor,” a value that provides additional coded information when TSVAL is
null. For example, for TSPARM = "AGEMAX" (Planned Maximum Age of Subjects), there is no value if a study does not specify a maximum age. In this case, the appropriate null flavor is "PINF", which stands for "positive infinity." In a clinical pharmacology study conducted in healthy volunteers for a drug where indications are not yet established, the appropriate null flavor for TSPARM = "INDIC" (Trial Disease/Condition Indication) would be "NA" (i.e., not applicable). TSVALNF can also be used in a case where the value of a particular parameter is unknown.

| Variable<br>Name | Variable Label | Type | Controlled<br>Terms, Codelist<br>or Format1 | Role | CDISC Notes | Core |
| --- | --- | --- | --- | --- | --- | --- |
| TSVALNF | Parameter<br>Value Null<br>Flavor | Char | ISO 21090<br>NullFlavor | Result<br>Qualifier | Null flavor for the value of TSPARM, to be populated only | Perm |
|  |  |  |  |  | if TSVAL is null. |  |
| TSVALCD | Parameter<br>Value Code | Char | * | Result<br>Qualifier | This is the code of the term in TSVAL. For example, | Exp |
|  |  |  |  |  | "6CW7F3G59X" is the code for gabapentin; "C49488" is |  |
|  |  |  |  |  | the code for Y. The length of this variable can be longer |  |
|  |  |  |  |  | than 8 to accommodate the length of the external |  |
|  |  |  |  |  | terminology. |  |
| TSVCDREF | Name of the<br>Reference<br>Terminology | Char | (DICTNAM) | Result<br>Qualifier | The name of the reference terminology from which | Exp |
|  |  |  |  |  | TSVALCD is taken. For example; CDISC CT, SNOMED, |  |
|  |  |  |  |  | ISO 8601. |  |
| TSVCDVER | Version of the<br>Reference<br>Terminology | Char |  | Result<br>Qualifier | The version number of the reference terminology, if | Exp |
|  |  |  |  |  | applicable. |  |


## Page 419

11. Some codelists used for TSVAL include terms which are also null flavors. For example, the Pharmaceutical
Dosage Form codelist includes the values "UNKNOWN" and "NOT APPLICABLE". In such cases, TSVAL should have the term from the codelist and TSVALNF should be null.
12. For some trials, there will be multiple records in the TS dataset for a single parameter. For example, a trial
that addresses both safety and efficacy could have 2 records with TSPARMCD = "TTYPE" (Trial Type), one with the TSVAL = "SAFETY" and the other with TSVAL = "EFFICACY". TSSEQ has a different value for each record for the same parameter. Note that this is different from datasets that contain subject data, where the --SEQ variable has a different value for each record for the same subject.
13. TS does not contain subject-level data, so there is no restriction analogous to the requirement in subjectlevel datasets that the blocks bound by TSGRPID are within a subject. TSGRPID can be used to tie
together any block of records in the dataset. TSGRPID is most likely to be used when the TS dataset includes multiple records for the same parameter. For example, if a trial compared administration of a total daily dose given once a day to that dose split over
2 administrations, the TS dataset might include the following records. There are 2 records each for
TSPARMCD = "Dose" and TSPARMCD = "DOSFREQ". Records with the same TSGRPID are associated with each other. In this example, dose units are the same for both administration schedules, so only 1 record for DOSU is needed.

14. Protocols vary in how they describe objectives. If the protocol does not provide information about which
objectives meet the definition of TSPARM = "OBJPRIM" (Trial Primary Objective; i.e., the principal purpose of the trial), then the objectives should be provided as values of TSPARM = "OBJPRIM". Consult the controlled terminology for trial summary parameters for appropriate parameter values for representing other objective designations (e.g., secondary, exploratory).
15. As per the definitions, the primary outcome measure is associated with the primary objective, the secondary
outcome measure is associated with the secondary objective, and the exploratory outcome measure is associated with the exploratory objective. It is possible for the same outcome measure to be associated with more than 1 objective. For example, 2 objectives could use the same outcome measure at different time points, or using different analysis methods.
16. If a primary objective is assessed by means of multiple outcome measures, then all of these outcome
measures should be provided as values of TSPARM = "OUTMSPR" (Primary Outcome Measure). Similarly, all outcome measures used to assess secondary objectives should be provided as values of TSPARM = "OUTMSSEC" (Secondary Outcome Measure), and all outcome measures used to assess exploratory objectives should be provided as values of TSPARM = "OUTMSEXP" (Exploratory Outcome Measure). Additional key measures of a study that are not designated as primary, secondary, or exploratory should be provided as values of TSPARM = "OUTMSADD" (Additional Outcome Measure).
17. Trial indication: Values for TSVAL when TSPARMCD = "INDIC" would indicate the condition, disease,
or disorder the trial is intended to investigate or address. A vaccine study of healthy subjects, with the intended purpose of preventing influenza infection, would have TSVAL = "Influenza". A clinical pharmacology study of healthy volunteers, with the purpose of collecting pharmacokinetic data, would have no trial indication; TSVAL would be null and TSVALNF = "NA" if TS contains a row where TSPARMCD = "INDIC".
18. Values for TSVAL when TSPARMCD = "REGID" (Registry Identifier) will be identifiers assigned by the
registry (e.g., ClinicalTrials.gov, EudraCT).

| TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL |
| --- | --- | --- | --- | --- |
| 1 | A | DOSE | Dose per Administration | 50 |
| 1 | A | DOSFREQ | Dosing Frequency | BID |
| 2 | B | DOSE | Dose per Administration | 100 |
| 2 | B | DOSFREQ | Dosing Frequency | Q24H |
| 1 |  | DOSU | Dose Units | mg |


## Page 420

TS – Examples Example 1 This example shows a subset of published controlled terminology parameters and the relationship of values across response variables TSVAL, TSVALNF, TSVALCD, TSVCDREF, and TSVCDVER. ts.xpt

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF | TSVCDVER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | XYZ | TS | 1 |  | ADDON | Added on to Existing<br>Treatments | Y |  | C49488 | CDISC CT | 2011-06-10 |
| 2 | XYZ | TS | 1 |  | AGEMAX | Planned Maximum Age<br>of Subjects | P70Y |  |  | ISO 8601 |  |
| 3 | XYZ | TS | 1 |  | AGEMIN | Planned Minimum Age<br>of Subjects | P18M |  |  | ISO 8601 |  |
| 4 | XYZ | TS | 1 |  | LENGTH | Trial Length | P3M |  |  | ISO 8601 |  |
| 5 | XYZ | TS | 1 |  | PLANSUB | Planned Number of<br>Subjects | 300 |  |  |  |  |
| 6 | XYZ | TS | 1 |  | RANDOM | Trial is Randomized | Y |  | C49488 | CDISC CT | 2011-06-10 |
| 7 | XYZ | TS | 1 |  | SEXPOP | Sex of Participants | BOTH |  | C49636 | CDISC CT | 2011-06-10 |
| 8 | XYZ | TS | 1 |  | STOPRULE | Study Stop Rules | INTERIM ANALYSIS FOR FUTILITY |  |  |  |  |
| 9 | XYZ | TS | 1 |  | TBLIND | Trial Blinding Schema | DOUBLE BLIND |  | C15228 | CDISC CT | 2011-06-10 |
| 10 | XYZ | TS | 1 |  | TCNTRL | Control Type | PLACEBO |  | C49648 | CDISC CT | 2011-06-10 |
| 11 | XYZ | TS | 1 |  | TDIGRP | Diagnosis Group | Neurofibromatosis Syndrome (Disorder) |  | 19133005 | SNOMED | 2011-03 |
| 12 | XYZ | TS | 1 |  | INDIC | Trial Disease/Condition<br>Indication | Tonic-Clonic Epilepsy (Disorder) |  | 352818000 | SNOMED | 2011-03 |
| 13 | XYZ | TS | 1 |  | TINDTP | Trial Intent Type | TREATMENT |  | C49656 | CDISC CT | 2011-06-10 |
| 14 | XYZ | TS | 1 |  | TITLE | Trial Title | A 24 Week Study of Oral Gabapentin vs. Placebo<br>as add-on Treatment to Phenytoin in Subjects with<br>Epilepsy due to Neurofibromatosis |  |  |  |  |
| 15 | XYZ | TS | 1 |  | TPHASE | Trial Phase<br>Classification | Phase II Trial |  | C15601 | CDISC CT | 2011-06-10 |
| 16 | XYZ | TS | 1 |  | TTYPE | Trial Type | EFFICACY |  | C49666 | CDISC CT | 2011-06-10 |
| 17 | XYZ | TS | 2 |  | TTYPE | Trial Type | SAFETY |  | C49667 | CDISC CT | 2011-06-10 |
| 18 | XYZ | TS | 1 |  | CURTRT | Current Therapy or<br>Treatment | Phenytoin |  | 6158TKW0C5 | UNII |  |
| 19 | XYZ | TS | 1 |  | OBJPRIM | Trial Primary Objective | Reduction in the 3-month seizure frequency from<br>baseline |  |  |  |  |
| 20 | XYZ | TS | 1 |  | OBJSEC | Trial Secondary<br>Objective | Percent reduction in the 3-month seizure<br>frequency from baseline |  |  |  |  |
| 21 | XYZ | TS | 2 |  | OBJSEC | Trial Secondary<br>Objective | Reduction in the 3-month tonic-clonic seizure<br>frequency from baseline |  |  |  |  |
| 22 | XYZ | TS | 1 |  | SPONSOR | Clinical Study Sponsor | Pharmaco |  | 123456789 | D-U-N-S<br>NUMBER |  |
| 23 | XYZ | TS | 1 |  | TRT | Investigational Therapy<br>or Treatment | Gabapentin |  | 6CW7F3G59X | UNII |  |
| 24 | XYZ | TS | 1 |  | RANDQT | Randomization Quotient | 0.67 |  |  |  |  |
| 25 | XYZ | TS | 1 |  | STRATFCT | Stratification Factor | SEX |  |  |  |  |
| 26 | XYZ | TS | 1 |  | REGID | Registry Identifier | NCT123456789 |  | NCT123456789 | ClinicalTrials.gov |  |
| 27 | XYZ | TS | 2 |  | REGID | Registry Identifier | XXYYZZ456 |  | XXYYZZ456 | EudraCT |  |
| 28 | XYZ | TS | 1 |  | OUTMSPRI | Primary Outcome<br>Measure | SEIZURE FREQUENCY |  |  |  |  |
| 29 | XYZ | TS | 1 |  | OUTMSSEC | Secondary Outcome<br>Measure | SEIZURE FREQUENCY |  |  |  |  |


## Page 421

Example 2 This example shows the relationship between parameters involving diagnosis and indication. Only selected trial summary parameters are included. Row 1: Shows the trial title. Row 2: Shows that subjects in this trial have a diagnosis of diabetes. Rows 3-4: Show the conditions with the intervention in the trial are intended to address. The 2 rows for the same parameter are differentiated by their TSSEQ values. Row 5: Shows that the intent of this trial is prevention of the conditions represented using the parameter "Trial Indication". ts.xpt

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF | TSVCDVER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 30 | XYZ | TS | 2 |  | OUTMSSEC | Secondary Outcome<br>Measure | SEIZURE DURATION |  |  |  |  |
| 31 | XYZ | TS | 1 |  | OUTMSEXP | Exploratory Outcome<br>Measure | SEIZURE INTENSITY |  |  |  |  |
| 32 | XYZ | TS | 1 |  | PCLAS | Pharmacological Class | Anti-epileptic Agent |  | N0000175753 | MED-RT |  |
| 33 | XYZ | TS | 1 |  | FCNTRY | Planned Country of<br>Investigational Sites | USA |  |  | ISO 3166-1<br>Alpha-3 |  |
| 34 | XYZ | TS | 2 |  | FCNTRY | Planned Country of<br>Investigational Sites | CAN |  |  | ISO 3166-1<br>Alpha-3 |  |
| 35 | XYZ | TS | 3 |  | FCNTRY | Planned Country of<br>Investigational Sites | MEX |  |  | ISO 3166-1<br>Alpha-3 |  |
| 36 | XYZ | TS | 1 |  | ADAPT | Adaptive Design | N |  | C49487 | CDISC CT | 2011-06-10 |
| 37 | XYZ | TS | 1 | PA | DCUTDTC | Data Cutoff Date | 2010-04-10 |  |  | ISO 8601 |  |
| 38 | XYZ | TS | 1 | PA | DCUTDESC | Data Cutoff Description | PRIMARY ANALYSIS |  |  |  |  |
| 39 | XYZ | TS | 1 |  | INTMODEL | Intervention Model | PARALLEL |  | C82639 | CDISC CT | 2011-06-10 |
| 40 | XYZ | TS | 1 |  | NARMS | Planned Number of<br>Arms | 3 |  |  |  |  |
| 41 | XYZ | TS | 1 |  | STYPE | Study Type | INTERVENTIONAL |  | C98388 | CDISC CT | 2011-06-10 |
| 42 | XYZ | TS | 1 |  | INTTYPE | Intervention Type | DRUG |  | C1909 | CDISC CT | 2011-06-10 |
| 43 | XYZ | TS | 1 |  | SSTDTC | Study Start Date | 2009-03-11 |  |  | ISO 8601 |  |
| 44 | XYZ | TS | 1 |  | SENDTC | Study End Date | 2011-04-01 |  |  | ISO 8601 |  |
| 45 | XYZ | TS | 1 |  | ACTSUB | Actual Number of<br>Subjects | 304 |  |  |  |  |
| 46 | XYZ | TS | 1 |  | HLTSUBJI | Healthy Subject<br>Indicator | N |  | C49487 | CDISC CT | 2011-06-10 |
| 47 | XYZ | TS | 1 |  | SDMDUR | Stable Disease<br>Minimum Duration | P3W |  |  | ISO 8601 |  |
| 48 | XYZ | TS | 1 |  | CRMDUR | Confirmed Response<br>Minimum Duration | P28D |  |  | ISO 8601 |  |

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF | TSVCDVER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | XYZ | TS | 1 |  | TITLE | Trial Type | A Study Comparing Cardiovascular Effects of Ticagrelor Versus<br>Placebo in Patients With Type 2 Diabetes Mellitus (THEMIS) |  |  |  |  |
| 2 | XYZ | TS | 1 |  | TDIGRP | Diagnosis<br>Group | Diabetes mellitus type 2 |  | 44054006 | SNOMED | 2017-03 |
| 3 | XYZ | TS | 1 |  | INDIC | Trial<br>Indication | Cardiac infarction |  | 22298006 | SNOMED | 2017-03 |
| 4 | XYZ | TS | 2 |  | INDIC | Trial<br>Indication | Cerebrovascular accident |  | 230690007 | SNOMED | 2017-01 |


## Page 422

Example 3 This example shows how to implement the null flavor in TSVALNF when the value in TSVAL is missing. Note that when TSVAL is null, TSVALCD is also null, and no code system is specified in TSVCDREF and TSVCDVER. Row 1: Shows that there was no upper limit on planned age of subjects, as indicated by TSVALNF="PINF" (the null value that means "positive infinity"). Row 2: Shows that trial phase classification is not applicable, as indicated by TSVALNF="NA". ts.xpt

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF | TSVCDVER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 5 | XYZ | TS | 1 |  | TINDTP | Trial Intent<br>Type | PREVENTION |  | C49657 | CDISC CT | 2017-03-01 |

Example 4 This example shows use of TSGRPID to group parameter values describing specific study parts (e.g., PHASE 1B, PHASE 3) and specific study treatments (e.g., DRUG X, DRUG Z). Rows 1-6: Show parameters and values that apply to the whole trial (i.e., both Phase 1B and Phase 3 parts of the trial). TSGRPID is null for this set of parameters. Rows 7-17: Show parameters and values that describe the Phase 1B part of the trial. TSGRPID is populated with a value of "PHASE 1B" for this set of parameters. Rows 18-29: Show parameters and values that describe the Phase 3 part of the trial. TSGRPID is populated with a value of "PHASE 3" for this set of parameters. Rows 30-33: Show parameters and values that describe details about 1 of the treatments planned in the trial. TSGRPID="DRUG X" for this set of parameters. Rows 34-37: Show parameters and values that describe details about 1 of the treatments planned in the trial. TSGRPID="DRUG Z" for this set of parameters. ts.xpt

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | XYZ | TS | 1 |  | AGEMAX | Planned Maximum Age of Subjects |  | PINF |  |  |
| 2 | XYZ | TS | 1 |  | TPHASE | Trial Phase Classification |  | NA |  |  |

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF | TSVCDVER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ABC123 | TS | 1 |  | TITLE | Trial Title | A Phase 1b/3, Multicenter Trial of Drug Z in Combination with<br>Drug X for Treatment of Melanoma |  |  |  |  |
| 2 | ABC123 | TS | 1 |  | INDIC | Trial Indication | Malignant melanoma |  | 372244006 | SNOMED | 2018-09-01 |
| 3 | ABC123 | TS | 1 |  | SEXPOP | Sex of Participants | BOTH |  | C49636 | CDISC CT | 2018-12-21 |
| 4 | ABC123 | TS | 1 |  | AGEMIN | Planned Minimum<br>Age of Subjects | P18Y |  |  | ISO 8601 |  |
| 5 | ABC123 | TS | 1 |  | AGEMAX | Planned Maximum<br>Age of Subjects |  | PINF |  |  |  |
| 6 | ABC123 | TS | 1 |  | HLTSUBJI | Healthy Subject<br>Indicator | N |  | C49487 | CDISC CT | 2018-12-21 |
| 7 | ABC123 | TS | 1 | PHASE<br>1B | TPHASE | Trial Phase<br>Classification | PHASE IB TRIAL |  |  |  |  |
| 8 | ABC123 | TS | 1 | PHASE<br>1B | TBLIND | Trial Blinding<br>Schema | OPEN LABEL |  | C49659 | CDISC CT | 2018-12-21 |


## Page 423

| Row | STUDYID | DOMAIN | TSSEQ | TSGRPID | TSPARMCD | TSPARM | TSVAL | TSVALNF | TSVALCD | TSVCDREF | TSVCDVER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 9 | ABC123 | TS | 1 | PHASE<br>1B | TCNTRL | Control Type | NONE |  | C41132 | CDISC CT | 2018-12-21 |
| 10 | ABC123 | TS | 1 | PHASE<br>1B | TTYPE | Trial Type | SAFETY |  | C49667 | CDISC CT | 2018-12-21 |
| 11 | ABC123 | TS | 1 | PHASE<br>1B | INTMODEL | Intervention Model | SINGLE GROUP |  | C82640 | CDISC CT | 2018-12-21 |
| 12 | ABC123 | TS | 1 | PHASE<br>1B | NARMS | Planned Number of<br>Arms | 1 |  |  |  |  |
| 13 | ABC123 | TS | 1 | PHASE<br>1B | PLANSUB | Planned Number of<br>Subjects | 30 |  |  |  |  |
| 14 | ABC123 | TS | 1 | PHASE<br>1B | RANDOM | Trial is Randomized | N |  | C49487 | CDISC CT | 2018-12-21 |
| 15 | ABC123 | TS | 1 | PHASE<br>1B | OBJPRIM | Trial Primary<br>Objective | To evaluate the safety, as assessed by incidence of dose<br>limiting toxicity, of combination therapy (Drug X + Drug Z) |  |  |  |  |
| 16 | ABC123 | TS | 1 | PHASE<br>1B | OUTMEAS | Primary Outcome<br>Measure | Incidence of dose limiting toxicities |  |  |  |  |
| 17 | ABC123 | TS | 1 | PHASE<br>1B | COMPTRT | Comparative<br>Treatment |  | NA |  |  |  |
| 18 | ABC123 | TS | 1 | PHASE 3 | TPHASE | Trial Phase<br>Classification | PHASE III TRIAL |  | C15602 | CDISC CT | 2018-12-21 |
| 19 | ABC123 | TS | 1 | PHASE 3 | TBLIND | Trial Blinding<br>Schema | DOUBLE BLIND |  | C15228 | CDISC CT | 2018-12-21 |
| 20 | ABC123 | TS | 1 | PHASE 3 | TCNTRL | Control Type | PLACEBO |  | C49648 | CDISC CT | 2018-12-21 |
| 21 | ABC123 | TS | 1 | PHASE 3 | TTYPE | Trial Type | EFFICACY |  | C49666 | CDISC CT | 2018-12-21 |
| 22 | ABC123 | TS | 1 | PHASE 3 | INTMODEL | Intervention Model | PARALLEL |  | C82639 | CDISC CT | 2018-12-21 |
| 23 | ABC123 | TS | 1 | PHASE 3 | NARMS | Planned Number of<br>Arms | 2 |  |  |  |  |
| 24 | ABC123 | TS | 1 | PHASE 3 | PLANSUB | Planned Number of<br>Subjects | 500 |  |  |  |  |
| 25 | ABC123 | TS | 1 | PHASE 3 | RANDOM | Trial is Randomized | Y |  | C49488 | CDISC CT | 2018-12-21 |
| 26 | ABC123 | TS | 1 | PHASE 3 | RANDQT | Randomization<br>Quotient | 0.5 |  |  |  |  |
| 27 | ABC123 | TS | 1 | PHASE 3 | OBJPRIM | Trial Primary<br>Objective | To evaluate the efficacy of combination therapy (Drug X + Drug<br>Z) versus monotherapy (Drug X + Placebo), as assessed by<br>progression-free survival using RECIST 1.1 |  |  |  |  |
| 28 | ABC123 | TS | 1 | PHASE 3 | OUTMEAS | Primary Outcome<br>Measure | Progression Free Survival (response evaluation by blinded<br>central review using RECIST 1.1) |  |  |  |  |
| 29 | ABC123 | TS | 1 | PHASE 3 | COMPTRT | Comparative<br>Treatment | DRUG X |  |  |  |  |
| 30 | ABC123 | TS | 1 | DRUG X | DOSE | Dose per<br>Administration | 200 |  |  |  |  |
| 31 | ABC123 | TS | 1 | DRUG X | DOSU | Dose Units | mg |  | C28253 | CDISC CT | 2018-12-21 |
| 32 | ABC123 | TS | 1 | DRUG X | DOSFRQ | Dosing Frequency | EVERY WEEK |  | C67069 | CDISC CT | 2018-12-21 |
| 33 | ABC123 | TS | 1 | DRUG X | ROUTE | Route of<br>Administration | ORAL |  | C38288 | CDISC CT | 2018-12-21 |
| 34 | ABC123 | TS | 1 | DRUG Z | DOSE | Dose per<br>Administration | 10000 |  |  |  |  |
| 35 | ABC123 | TS | 1 | DRUG Z | DOSU | Dose Units | PFU |  | C67264 | CDISC CT | 2018-12-21 |
| 36 | ABC123 | TS | 1 | DRUG Z | DOSFRQ | Dosing Frequency | EVERY 2 WEEKS |  | C71127 | CDISC CT | 2018-12-21 |
| 37 | ABC123 | TS | 1 | DRUG Z | ROUTE | Route of<br>Administration | INTRATUMOR |  | C38269 | CDISC CT | 2018-12-21 |


## Page 424

7.4.2.1 Use of Null Flavor
The variable TSVALNF is based on the idea of a “null flavor” as embodied in the ISO 21090 standard (Health Informatics – Harmonized data types for information exchange; https://www.iso.org/standard/35646.html). A null flavor is an ancillary piece of data that provides additional information when its primary piece of data is null (has a missing value). There is controlled terminology for the null flavor data item which includes such familiar values as "Unknown", "Other", and "Not Applicable" among its 14 terms. The proposal to include a null flavor variable to supplement the TSVAL variable in the Trial Summary Information (TS) dataset arose when it was realized that the TS model did not have a good way to represent the fact that a protocol placed no upper limit on the age of study subjects. When the trial summary parameter is AGEMAX, then TSVAL should have a value expressed as an ISO 8601 time duration (e.g., P43Y for 43 years old, P6M for 6 months old). Although it would be possible to allow a value such as "NONE" or "UNBOUNDED" to be entered in TSVAL, validation programs would then have to recognize this special term as an exception to the expected data format. Therefore, the SDS team decided that a separate null flavor variable that uses the ISO 21090 null-flavor terminology would be a better solution. The SDS Team also decided to specify the use of a null-flavor variable in the TS domain with SDTMIG v3.4 as a way of testing the use of such a variable in a limited setting. As the title of ISO 21090 suggests, that standard was developed for use with healthcare data; it is expected that it will eventually see wide use in the clinical data from which clinical trial data are derived. CDISC already uses this data-type standard (see BRIDG; https://www.cdisc.org/standards/). The null flavor, in particular, is a solution to the widespread problem of needing or wanting to convey information that will help in the interpretation of a missing value. Although null flavors could certainly be eventually used for this purpose in other cases (e.g., with subject data), doing so at this time would be extremely disruptive and premature. The use of null flavors for the variable TSVAL provides an opportunity for sponsors and reviewers to learn about the null flavors and to evaluate their usefulness in a concrete setting. The controlled terminology for null flavor, which supersedes Appendix C1, Supplemental Qualifiers Name Codes, is included below.

| NullFlavor Enumeration. OID: 2.16.840.1.113883.5.1008 |  |  |  |
| --- | --- | --- | --- |
| 1 | NI | No information | The value is exceptional (i.e., missing, omitted, incomplete, improper). No information as to the reason<br>for being an exceptional value is provided. This is the most general exceptional value. It is also the<br>default exceptional value. |
| 2 | INV | Invalid | The value as represented in the instance is not a member of the set of permitted data values in the<br>constrained value domain of a variable. |
| 3 | OTH | Other | The actual value is not a member of the set of permitted data values in the constrained value domain<br>of a variable (e.g., concept not provided by required code system). |
| 4 | PINF | Positive infinity | Positive infinity of numbers |
| 4 | NINF | Negative<br>infinity | Negative infinity of numbers |
| 3 | UNC | Unencoded | No attempt has been made to encode the information correctly, but the raw source information is<br>represented (usually in original Text). |
| 3 | DER | Derived | An actual value may exist, but it must be derived from the information provided (usually an expression<br>is provided directly). |
| 2 | UNK | Unknown | A proper value is applicable, but not known. |
| 3 | ASKU | Asked but<br>unknown | Information was sought but not found (e.g., patient was asked but didn't know). |
| 4 | NAV | Temporarily<br>unavailable | Information is not available at this time, but is expected to be available later. |
| 3 | NASK | Not asked | This information has not been sought (e.g., patient was not asked). |
| 3 | QS | Sufficient<br>quantity | The specific quantity is not known, but is known to be non-zero and is not specified because it makes<br>up the bulk of the material. For example, if directions said, "Add 10 mg of ingredient X, 50 mg of<br>ingredient Y, and sufficient quantity of water to 100 ml", the null flavor "QS" would be used to express<br>the quantity of water. |
| 3 | TRC | Trace | The content is greater than zero, but too small to be quantified. |
| 2 | MSK | Masked | There is information on this item available, but it has not been provided by the sender due to security,<br>privacy or other reasons. There may be an alternate mechanism for gaining access to this information. |


## Page 425

The numbers in column 1 of the table describe the hierarchy of these values:
• No information
o Invalid
▪ Other
• Positive infinity
• Negative infinity
▪ Unencoded ▪ Derived
o Unknown
▪ Asked but unknown
• Temporarily unavailable
▪ Not asked ▪ Quantity sufficient ▪ Trace
o Masked
o Not applicable
The 1 value at level 1 (No information) is the least informative. It merely confirms that the primary piece of data is null. The values at level 2 provide a little more information, distinguishing between situations where the primary piece of data is not applicable and those where it is applicable but masked, unknown, or invalid (i.e., not in the correct format to be represented in the primary piece of data). The values at levels 3 and 4 provide successively more information about the situation. For example, for the MAXAGE case that provided the impetus for the creation of the TSVALNF variable, the value PINF means that there is information about the maximum age, but it is not something that can be expressed, as in the ISO8601 quantity of time format required for populating TSVAL. The null flavor PINF provides the most complete information possible in this case (i.e., that the maximum age for the study is unbounded).
7.5 How to Model the Design of a Clinical Trial
The following steps allow the modeler to move from more-familiar concepts, such as arms, to less-familiar concepts, such as elements and epochs. The actual process of modeling a trial may depart from these numbered steps. Some steps will overlap; there may be several iterations; and not all steps are relevant for all studies.
1. Start from the flow chart or schema diagram usually included in the trial protocol. This diagram will show
how many arms the trial has, and the branch points or decision points where the arms diverge.
2. Write down the decision rule for each branching point in the diagram. Does the assignment of a subject to
an arm depend on a randomization? On whether the subject responded to treatment? On some other criterion?
3. If the trial has multiple branching points, check whether all the branches that have been identified really
lead to different arms. The arms will relate to the major comparisons the trial is designed to address. For

| NullFlavor Enumeration. OID: 2.16.840.1.113883.5.1008 |  |  |  |
| --- | --- | --- | --- |
|  |  |  | WARNING — Use of this null flavor does provide information that may be a breach of<br>confidentiality, even though no detailed data are provided. Its primary purpose is for those<br>circumstances where it is necessary to inform the receiver that the information does exist<br>without providing any detail. |
| 2 | NA | Not applicable | No proper value is applicable in this context (e.g., last menstrual period for a male). |


## Page 426

some trials, there may be a group of somewhat different paths through the trial that are all considered to belong to a single arm.
4. For each arm, identify the major time periods of treatment and non-treatment a subject assigned to that
arm will go through. These are the elements, or building blocks, of which the arm is composed.
5. Define the starting point of each element. Define the rule for how long the element should last. Determine
whether the element is of fixed duration.
6. Re-examine the sequences of elements that make up the various arms and consider alternative
element definitions. Would it be better to “split” some elements into smaller pieces or “lump” some elements into larger pieces? Such decisions will depend on the aims of the trial and plans for analysis.
7. Compare the various arms. In most clinical trials, especially blinded trials, the pattern of elements will be
similar for all arms, and it will make sense to define trial epochs. Assign names to these epochs. During the conduct of a blinded trial, it will not be known which arm a subject has been assigned to, or which treatment elements they are experiencing, but the epochs they are passing through will be known.
8. Identify the visits planned for the trial. Define the planned start timings for each visit, expressed relative to
the ordered sequences of elements that make up the arms. Define the rules for when each visit should end.
9. For oncology trials or other trials with disease assessments that are not necessarily tied to visits, find the
planned timing of disease assessments in the protocol and record it in the Trial Disease Assessments (TD) dataset.
10. If the protocol includes data collection that is triggered by the occurrence of certain events, interventions,
or findings, record those triggers in the Trial Disease Milestones (TM) dataset. Note that disease milestones may be pre- (e.g., disease diagnosis) or on-study.
11. Identify the inclusion and exclusion criteria to be able to populate the Trial Inclusion/Exclusion Criteria
(TI) dataset. If inclusion and exclusion criteria were amended so that subjects entered under different versions, populate TIVERS to represent the different versions.
12. Populate the TS dataset with summary information.


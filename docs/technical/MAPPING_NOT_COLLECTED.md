## Define-XML Integration (Future)

The `not_collected` map stores variable → reason pairs that will be used when
generating Define-XML:

```xml
<ItemDef OID="IT.DM.RFSTDTC" Name="RFSTDTC" ...>
<Description>
<TranslatedText xml:lang="en">Reference Start Date/Time</TranslatedText>
</Description>
        <!-- Comment from not_collected reason -->
<def:CommentOID OID="COM.DM.RFSTDTC"/>
        </ItemDef>

<def:CommentDef OID="COM.DM.RFSTDTC">
<Description>
    <TranslatedText xml:lang="en">Data not collected in this study</TranslatedText>
</Description>
</def:CommentDef>
```

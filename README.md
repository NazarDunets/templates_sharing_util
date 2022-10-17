# templates_util

### !Only works for OS X!

## Requirements
Create a folder with template files you want to share and file named `shared_templates.xml` with contents like

```xml
<shared_templates>
    <!-- your templates -->
</shared_templates>
```

### Example
Folder
```
folder_with_templates
	|- shared_templates.xml
	|- SomeTemplate.kt
```
`shared_templates.xml`
```xml
<shared_templates>
    <template name="Node.kt" file-name="${Name}Node" reformat="true" live-template-enabled="false"/>
    <template name="SimpleNIF.kt" file-name="${Name}Feature" reformat="true" live-template-enabled="false">
        <template name="SimpleNIF.kt.child.0.kt" file-name="${Name}Node" reformat="true" live-template-enabled="false" />
        <template name="SimpleNIF.kt.child.1.kt" file-name="${Name}Interactor" reformat="true" live-template-enabled="false" />
    </template>
</shared_templates>
```
### Where to look at my current templates?
If you used default installation path for AS, you can find your AndroidStudio templates config at 
`$HOME/Library/Application\ Support/Google/YOUR_AS_FOLDER/options/file.template.settings.xml`

## Running

```sh
templates_util folder_with_templates
```

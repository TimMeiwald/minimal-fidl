<ws> Inline = (<multiline_comment>/<comment>/' '/'\t'/'\r')*;
<wsn> Inline = (<multiline_comment>/<comment>/' '/'\t'/'\r'/'\n')*;
<ws_atlone> Inline = (' '/'\t')+;
<wsn_nocomment> Inline = (' '/'\t'/'\r'/'\n')*;
<ascii> Inline = [0x00..0xFF];
<multiline_comment> = "/*", (!"*/", <ascii>)*, "*/";
<comment> = "//", (!'\n', <ascii>)*;

<digit> Inline = ['0'..'9'];
<digits> = <digit>+;
<integer> = <sign>, <ws>, <digits>, <exponent>?;
<float> = <sign>, <ws>, <digits>, <fraction>, <exponent>?;
<fraction> = ('.', <digits>);
<exponent> = <ws>, ('E'/'e'), <ws>, <integer>;
<sign> = ('+'/'-')?;

<hex_char> Inline = <digit>/['A'..'F']/['a'..'f'];
<hex> = "0x", <hex_char>+;

<bin_char> Inline = '0'/'1';
<binary> = "0b", <bin_char>+; 

<number> = <hex>/<binary>/<float>/<integer>;

<annotation_block> = "<**", <wsn_nocomment>, (!"**>", <annotation>, <wsn_nocomment>)+, "**>";
<annotation> = '@', <annotation_name>, <wsn_nocomment>, ':', <annotation_content>;
<annotation_content> = (!'@', !"**>",<ascii>)*; # You cannot have comments/multiline comments inside an annotation#
<annotation_name> = <type_char>+; #type char because same semantically and inlined anyway#

<type_char> Inline = '_'/['A'..'Z']/['a'..'z'] ;
<type_char_with_num> Inline = <digit>/<type_char>;
<type_name> Inline = <type_char>, <type_char_with_num>*;
<type_dec> = <type_name>;
<array> = <ws>,'[', <ws>,']';
<type_ref> = <type_name>, ('.', <type_name>)*, <array>?;
<variable_name> = <type_name>; 


<file_path> = '"', (!'"', <ascii>)* ,'"';
<wildcard> = ".*";

<package> = "package", <ws_atlone>, <type_ref>, <ws>; #Describes the package import#
<import_namespace> = "import" , <ws_atlone>, <type_ref>, <wildcard>, <ws_atlone>, "from", <ws_atlone>, <file_path>, <ws>;
<import_model> = "import", <ws_atlone>,  "model", <ws_atlone>, <file_path> ,<ws>;

<attribute> =   <annotation_block>?, <wsn>,
                "attribute", <ws_atlone>, 
                <type_ref>, <ws_atlone>, 
                <variable_name>, <wsn>;
<variable_declaration> = <annotation_block>?, <wsn>, 
                        <type_ref>, <wsn>, 
                        <variable_name>, <wsn>;
<input_params> = <annotation_block>?, <wsn>,
                "in", <wsn>, 
                '{', 
                <wsn>, 
                <variable_declaration>*, 
                '}';
<output_params> = <annotation_block>?, <wsn>,
                "out", <wsn>, 
                '{', 
                <wsn>,
                <variable_declaration>*, 
                '}';
<method> =  <annotation_block>?, <wsn>, 
            "method", <wsn>, 
            <variable_name>, <wsn>, 
            '{', <wsn>, 
            <input_params>?, <wsn>, 
            <output_params>?, <wsn>,
            '}';

<typedef> = <annotation_block>?, <wsn>, 
            "typedef", <ws>,
            <type_dec>, <ws>, 
            "is", <ws>, 
            <type_ref>;

<structure> =   <annotation_block>?, <wsn>, 
                "struct", <ws>, 
                <type_dec>, <wsn>,
                '{', <wsn>, 
                <variable_declaration>*,
                '}';

<enumeration> = <annotation_block>?, <wsn>, 
                "enumeration", <ws>, 
                <type_dec>, <wsn>, 
                '{', 
                <wsn>,
                (<enum_value>, <wsn>)*, 
                '}';
<enum_value> =  <annotation_block>?, <wsn>, 
                <variable_name>, <ws>, 
                ('=', <ws>, <number>)?, 
                <ws>, ','?;
<version> = "version", <wsn>, 
            '{', <wsn>, 
            <major>, <wsn>, 
            <minor>, <wsn>, '}';
<major> = "major", <ws>, <digits>;
<minor> = "minor", <ws>, <digits>;
<interface> = <annotation_block>?, <wsn>, "interface", <wsn>, 
                <variable_name>, <wsn>, 
                '{', <wsn>, 
                <version>?, <wsn>, 
                ((<method>/<typedef>/<structure>/<attribute>/<enumeration>), <wsn>)*, 
                <wsn>, '}';
<type_collection> = <annotation_block>?, <wsn>, 
                    "typeCollection", <ws>, 
                    <variable_name>?, <wsn>, 
                    '{', <wsn>, <version>?, <wsn>,
                    ((<typedef>/<structure>/<enumeration>), <wsn>)*,
                    '}';
<Grammar> = <wsn>, <package>, 
            <wsn>, ((<import_model>/<import_namespace>), <wsn>)*, 
            <wsn>, ((<interface>/<type_collection>), <wsn>)*, 
            <wsn>;

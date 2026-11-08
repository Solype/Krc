# The rule "a name cannot end in 다" goes: a declared name in 다 gives no warning either.
for program in bare_name_in_expression bare_name_in_condition parameter_ending_in_da \
    verb_next_to_a_name_in_da; do
    check "no_warning_$program" silent "$program.kr"
done
check "rule_gone_from_the_message" source_lacks 'a name cannot end in 다' src/parser/preparser

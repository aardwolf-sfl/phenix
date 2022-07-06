#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>

#include "generated.c"

int main(int argc, char const *argv[])
{
    FILE *output;
    int error;

    if (argc > 1)
    {
        output = fopen(argv[1], "w");

        if (output == NULL)
        {
            error = errno;
            perror("cannot open file for output");
            return error;
        }
    }
    else
    {
        output = stdout;
    }

    phenix_generated_person_t person;
    person.name = "Felix";
    person.age = 42;
    person.pronouns.subject = phenix_generated_nested_pronoun_they;
    person.pronouns.object = phenix_generated_nested_pronoun_them;
    person.degree.tag_ = phenix_generated_degree_highest;
    person.degree.highest.name = phenix_generated_nested_degree_name_master;

    phenix_generated_nested_country_init(&person.citizenship);
    phenix_generated_nested_country_set(&person.citizenship, PHENIX_GENERATED_NESTED_COUNTRY_CZECH_REPUBLIC);
    phenix_generated_nested_country_set(&person.citizenship, PHENIX_GENERATED_NESTED_COUNTRY_FRANCE);

    bool working_hours[] = {false, false, false, false, false, false, false, false, false, true, true, true, true,
                            true, true, true, true, false, false, false, false, false, false, false};

    phenix_generated_vector_bool_init(&person.working_hours, working_hours, sizeof(working_hours));

    phenix_generated_person_encode(&person, output);

    phenix_generated_project_t project;
    project.name = "Rust";
    project.url = "https://github.com/rust-lang/rust";
    phenix_generated_stream_project_push_encode(&project, output);

    project.name = "Linux";
    project.url = "https://github.com/torvalds/linux";
    phenix_generated_stream_project_push_encode(&project, output);

    project.name = "Phenix";
    project.url = "https://github.com/aardwolf-sfl/phenix";
    phenix_generated_stream_project_push_encode(&project, output);

    return 0;
}

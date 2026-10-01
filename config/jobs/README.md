# jobs

[job-folder](../../src/jobs/job-folder) workflows, one folder each. The
Deployfile links every file here into a real `~/jobs/<name>/` folder and
creates the queue folders beside it:

```
config/jobs/<name>/job.sh   tracked here     -> ~/jobs/<name>/job.sh (symlink)
~/jobs/<name>/input/        created by deploy, never linked, never tracked
~/jobs/<name>/output/       "
~/jobs/<name>/done/         "
~/jobs/<name>/failed/       "
~/jobs/<name>/*.log         written by job-folder, never tracked
```

To add one: create `config/jobs/<name>/job.sh`, `chmod +x` it, and run
`scripts/setup/deploy.sh`. The script gets `$INPUT`, `$INPUT_DIR`,
`$INPUT_FILE`, `$INPUT_NAME` and `$OUTPUT_DIR`; see the job-folder README for
the rest. Helpers the script needs can sit beside it and are linked too.

Workflows that tools generate on the fly (`topaz-job`, `interpolate-resolve-job`)
are not tracked here: they write their own `job.sh` straight into `~/jobs`.
This file is not deployed.

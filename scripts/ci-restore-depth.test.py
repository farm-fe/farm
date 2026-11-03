"""Real Git shallow merge rejection and full-history native restoration contract."""
# cspell:words importlib NOSYSTEM
import importlib.util
import os
from pathlib import Path
import subprocess
import unittest
from unittest import mock

spec=importlib.util.spec_from_file_location('restore_depth_native_fixture',Path(__file__).with_name('ci-leaf-native.test.py'))
fixtures=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixtures)
live=fixtures.live

class Contracts(unittest.TestCase):
    def test_actual_shallow_merge_loses_parents_and_full_history_restores_same_original_bytes(self):
        fixture=fixtures.Contracts();fixture.setUp()
        source_dir=fixture.root/'git-source';source_dir.mkdir()
        empty=fixture.root/'empty-git-config';empty.write_text('')
        environment={key:value for key,value in os.environ.items() if not key.startswith('GIT_')}
        environment.update(GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL=str(empty),GIT_AUTHOR_NAME='Fixture',GIT_AUTHOR_EMAIL='fixture@example.invalid',GIT_COMMITTER_NAME='Fixture',GIT_COMMITTER_EMAIL='fixture@example.invalid')
        real_run=subprocess.run
        def git(directory,*args):
            return real_run(['git',*args],cwd=directory,env=environment,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=True,timeout=20).stdout
        git(source_dir,'init','--quiet','--initial-branch=main')
        for name,body in fixture.origin['files'].items():
            path=source_dir/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(body)
        git(source_dir,'add','.');git(source_dir,'commit','--quiet','-m','base')
        base=git(source_dir,'rev-parse','HEAD').decode().strip()
        git(source_dir,'checkout','--quiet','-b','topic')
        (source_dir/'docs/experiences/engineering.md').write_text('Topic engineering notes\n')
        git(source_dir,'add','.');git(source_dir,'commit','--quiet','-m','topic')
        head=git(source_dir,'rev-parse','HEAD').decode().strip()
        git(source_dir,'checkout','--quiet','main')
        (source_dir/'docs/experiences/main.md').write_text('Main engineering notes\n')
        git(source_dir,'add','.');git(source_dir,'commit','--quiet','-m','main')
        git(source_dir,'merge','--quiet','--no-ff','topic','-m','merge')
        merge=git(source_dir,'rev-parse','HEAD').decode().strip()
        def read(directory,sha):
            def isolated_run(command,**options):
                if command[0]=='git':options.update(cwd=directory,env=environment)
                return real_run(command,**options)
            with mock.patch.object(live.evidence.chain_plan.impact,'git',side_effect=lambda *args:git(directory,*args)), mock.patch.object(live.evidence.chain_plan.impact.subprocess,'run',side_effect=isolated_run):
                return live.evidence.source_reader(sha)
        fixture.origin=read(source_dir,base);fixture.current=read(source_dir,merge)
        self.assertIn(head,fixture.current['parents'])
        fixture.captured['originBuildSHA']=base;fixture.producer_env['GITHUB_SHA']=base
        fixture.environment['GITHUB_SHA']=merge;fixture.run['head_sha']=base;fixture.main=base
        fixture.manifest,fixture.bodies=live.create_bundle(fixture.origin,fixture.captured,fixture.root,fixture.producer_env);fixture.pack()
        transport,runner,commitment,raw,artifact,rows=fixture.rerun_materialization()
        original=transport.get.side_effect
        def get(route):
            value=original(route)
            return {**value,'head_sha':head} if '/attempts/' in route and '/jobs?' not in route else value
        transport.get.side_effect=get
        shallow=fixture.root/'shallow-source'
        git(fixture.root,'clone','--quiet','--depth','1','--branch','main',source_dir.as_uri(),str(shallow))
        shallow_source=read(shallow,merge)
        self.assertEqual(shallow_source['parents'],[])
        with self.assertRaisesRegex(ValueError,'Materialization current source differs'):
            live.consumer_producer(transport,shallow_source,runner,commitment)
        transport.archive.assert_not_called()
        restored=fixture.root/'full-history-restored';binding=live.restore_materialization(transport,fixture.current,runner,commitment,restored)
        destination=fixture.root/'current-consumer';destination.mkdir()
        with mock.patch.object(live.evidence.transport_module,'GitHubReadOnly',return_value=transport):
            receipt=live.consume(restored,commitment,fixture.current,runner,destination)
        self.assertEqual(binding['resolutionAttempt'],1);self.assertEqual(binding['consumerAttempt'],3)
        self.assertEqual(receipt['attempt'],1);self.assertEqual((restored/'consumer.json').read_bytes(),raw)
        self.assertEqual((destination/'packages/core/binding/farm.linux-x64-gnu.node').read_bytes(),fixture.native)

if __name__=='__main__':unittest.main(verbosity=2)

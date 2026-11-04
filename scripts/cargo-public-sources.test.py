# cspell:words itoa nitoa
"""Public Cargo twins: graph contracts and opt-in real Cargo/registry tests.

FARM_CARGO_NETWORK_TESTS=1 python -B scripts/cargo-public-sources.test.py
The integration suite downloads crates but never uploads or publishes anything.
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('cargo_sources', Path(__file__).with_name('cargo-source-release.py'))
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)


class SourceGraph(unittest.TestCase):
    def graph(self):
        rows = [('consumer', '1.0.0', None), ('leaf', '1.0.0', None),
                ('serde', '1.0.228', None), ('itoa', '1.0.15', 'registry+https://github.com/rust-lang/crates.io-index'),
                ('unrelated', '1.0.0', None)]
        return {'packages': [dict(id=name, name=name, version=version, source=source) for name, version, source in rows],
                'resolve': {'nodes': [dict(id=name, dependencies=deps, features=['std'] if name == 'serde' else [])
                                     for name, deps in [('consumer', ['leaf', 'itoa']), ('leaf', ['serde']),
                                                        ('serde', []), ('itoa', []), ('unrelated', [])]]}}

    def test_transitive_twins_and_retained_members_exclude_first_publication_and_unrelated(self):
        selected = [{'name': 'consumer', 'version': '1.0.0'}]
        self.assertEqual(m.public_source_seeds(self.graph(), selected), [
            {'name': 'leaf', 'version': '1.0.0', 'features': []},
            {'name': 'serde', 'version': '1.0.228', 'features': ['std']}])
        selected.append({'name': 'leaf', 'version': '1.0.0'})
        self.assertEqual([row['name'] for row in m.public_source_seeds(self.graph(), selected)], ['serde'])
        self.assertEqual(m.public_source_seeds(self.graph(), []), [])

    def test_missing_selected_version_or_incomplete_graph_fails_closed(self):
        with self.assertRaisesRegex(ValueError, 'exact selected'):
            m.public_source_seeds(self.graph(), [{'name': 'consumer', 'version': '2.0.0'}])
        graph = self.graph(); graph['resolve']['nodes'].pop(2)
        with self.assertRaisesRegex(ValueError, 'Incomplete'):
            m.public_source_seeds(graph, [{'name': 'consumer', 'version': '1.0.0'}])

    def test_isolated_exact_roots_and_locked_fetch(self):
        seen = []
        def cargo(command, cwd, env, check):
            self.assertTrue(check)
            text = (cwd / 'Cargo.toml').read_text()
            self.assertIn('[workspace]', text); self.assertNotIn('[patch', text)
            self.assertIn('registry = "crates-io"', text)
            self.assertIn('"=1.0.228"', text)
            self.assertEqual(env['RUSTUP_TOOLCHAIN'], 'pinned-toolchain')
            seen.append(command)
        with mock.patch.object(m, 'run', return_value='pinned-toolchain (overridden)'), mock.patch.object(m.subprocess, 'run', side_effect=cargo):
            m.fetch_public_sources([{'name': 'serde', 'version': '1.0.228', 'features': ['derive']}])
        self.assertEqual(seen, [['cargo', 'generate-lockfile'], ['cargo', 'fetch', '--locked']])


@unittest.skipUnless(os.environ.get('FARM_CARGO_NETWORK_TESTS') == '1', 'explicit read-only registry integration opt-in')
class RealCargo(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='farm-cargo-integration-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.workspace = self.root / 'workspace'; self.workspace.mkdir()
        self.env = {**os.environ, 'CARGO_HOME': str(self.root / 'cargo-home'),
                    'CARGO_TARGET_DIR': str(self.root / 'target'),
                    'RUSTUP_TOOLCHAIN': m.run('rustup', 'show', 'active-toolchain').split()[0]}
        self.selected = [{'name': 'farm_source_fixture_leaf', 'version': '0.0.1'},
                         {'name': 'farm_source_fixture_consumer', 'version': '0.0.1'}]

    def cargo(self, *args, succeeds=True):
        result = subprocess.run(['cargo', *args], cwd=self.workspace, env=self.env,
                                text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=180)
        if succeeds:
            self.assertEqual(result.returncode, 0, result.stdout)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result.stdout

    def crate(self, path, name, version, dependencies='', features='', source='pub fn value() -> u8 { 1 }'):
        root = self.workspace / path; (root / 'src').mkdir(parents=True)
        (root / 'Cargo.toml').write_text(f'[package]\nname="{name}"\nversion="{version}"\nedition="2024"\nlicense="MIT"\ndescription="Cargo source fixture"\n{dependencies}\n{features}\n')
        (root / 'src/lib.rs').write_text(source)

    def fixture(self, patch_itoa=False, bad_feature=False, optional_target=False):
        self.crate('serde', 'serde', '1.0.228', features='[features]\nlocal_only=[]' if bad_feature else '')
        if patch_itoa: self.crate('itoa', 'itoa', '1.0.15')
        features = ', features=["local_only"]' if bad_feature else ''
        itoa = ('\n[target."cfg(windows)".dependencies]\nitoa={version="=1.0.15",optional=true}\n[features]\nwith-itoa=["dep:itoa"]'
                if optional_target else '\nitoa="=1.0.15"')
        self.crate('leaf', 'farm_source_fixture_leaf', '0.0.1',
                   '[dependencies]\nserde={version="=1.0.228"' + features + '}' + itoa)
        self.crate('consumer', 'farm_source_fixture_consumer', '0.0.1',
                   '[dependencies]\nfarm_source_fixture_leaf={path="../leaf",version="=0.0.1"}',
                   source='pub fn value() -> u8 { farm_source_fixture_leaf::value() }')
        (self.workspace / 'Cargo.toml').write_text('[workspace]\nresolver="3"\nmembers=["leaf","consumer"]\nexclude=["serde","itoa"]\n[patch.crates-io]\nserde={path="serde"}\n' + ('itoa={path="itoa"}\n' if patch_itoa else ''))
        self.cargo('generate-lockfile'); self.cargo('fetch', '--locked')
        self.lock = (self.workspace / 'Cargo.lock').read_bytes()
        self.metadata = json.loads(self.cargo('metadata', '--locked', '--offline', '--all-features', '--format-version', '1'))

    def package(self, succeeds=True):
        return self.cargo('package', '--locked', '--offline', '--registry', 'crates-io',
                          '-p', self.selected[0]['name'], '-p', self.selected[1]['name'], succeeds=succeeds)

    def prefetch(self, seeds=None, offline=False):
        env = {**self.env, **({'CARGO_NET_OFFLINE': 'true'} if offline else {})}
        with mock.patch.dict(os.environ, env):
            m.fetch_public_sources(seeds if seeds is not None else m.public_source_seeds(self.metadata, self.selected))

    def test_empty_cache_patched_and_unpatched_dependencies_then_first_family_publication(self):
        self.fixture()
        self.assertIn('no matching package named `serde', self.package(succeeds=False))
        self.prefetch(); output = self.package()
        self.assertIn('Verifying farm_source_fixture_consumer', output)
        self.assertEqual((self.workspace / 'Cargo.lock').read_bytes(), self.lock)
        for row in self.selected:
            archive = self.root / 'target/package' / (row['name'] + '-' + row['version'] + '.crate')
            m.safe_archive(archive.read_bytes(), row['name'], row['version'])

    def test_multiple_patched_dependencies_are_discovered(self):
        self.fixture(patch_itoa=True, optional_target=True)
        self.assertEqual([row['name'] for row in m.public_source_seeds(self.metadata, self.selected)], ['itoa', 'serde'])
        self.prefetch(); self.package()

    def test_no_network_fails_before_offline_package_can_pass(self):
        self.fixture()
        with self.assertRaises(subprocess.CalledProcessError): self.prefetch(offline=True)
        self.package(succeeds=False)
        self.assertEqual((self.workspace / 'Cargo.lock').read_bytes(), self.lock)

    def test_conflicting_public_transitive_version_is_rejected_by_final_package(self):
        self.fixture()
        manifest = self.workspace / 'leaf/Cargo.toml'
        manifest.write_text(manifest.read_text() + '\nserde_core="=1.0.227"\n')
        self.cargo('generate-lockfile'); self.cargo('fetch', '--locked')
        self.lock = (self.workspace / 'Cargo.lock').read_bytes()
        self.metadata = json.loads(self.cargo('metadata', '--locked', '--offline', '--all-features', '--format-version', '1'))
        self.prefetch()
        output = self.package(succeeds=False)
        self.assertIn('serde_core', output)
        self.assertEqual((self.workspace / 'Cargo.lock').read_bytes(), self.lock)

    def test_public_version_or_feature_conflict_fails_closed(self):
        self.fixture(bad_feature=True)
        with self.assertRaises(subprocess.CalledProcessError): self.prefetch()
        with self.assertRaises(subprocess.CalledProcessError):
            self.prefetch([{'name': 'serde', 'version': '999.0.0', 'features': []}])
        self.assertEqual((self.workspace / 'Cargo.lock').read_bytes(), self.lock)


if __name__ == '__main__': unittest.main(verbosity=2)

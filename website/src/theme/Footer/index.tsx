import { DiscordLogoIcon, GitHubLogoIcon } from '@radix-ui/react-icons';
import React from 'react';
import { FaXTwitter } from 'react-icons/fa6';

export default function FooterWrapper() {
  return <footer className="farm-site-footer farm-editorial-footer">
    <div className="farm-editorial-footer-top">
      <div><img className="farm-original-footer-logo" src="/img/logo-farm.png" alt="Farm" /><p>Extremely Fast Web Build Tool Written in Rust</p></div>
      <nav aria-label="Footer navigation">
        <div><h2>Guide</h2><a href="/docs/quick-start">Quick Start</a><a href="/docs/why-farm">Introduction</a></div>
        <div><h2>Community</h2><a href="https://github.com/farm-fe/farm?tab=readme-ov-file#chat-with-us">WeChat Group</a><a href="https://discord.com/invite/mDErq9aFnF">Discord</a></div>
        <div><h2>More</h2><a href="https://github.com/farm-fe/farm">Github</a><a href="https://x.com/FarmFe71928">Twitter</a></div>
      </nav>
    </div>
    <nav className="farm-original-social" aria-label="Social links"><a href="https://github.com/farm-fe/farm" aria-label="GitHub" target="_blank" rel="noopener noreferrer"><GitHubLogoIcon aria-hidden="true" /></a><a href="https://x.com/FarmFe71928" aria-label="Twitter" target="_blank" rel="noopener noreferrer"><FaXTwitter aria-hidden="true" /></a><a href="https://discord.com/invite/mDErq9aFnF" aria-label="Discord" target="_blank" rel="noopener noreferrer"><DiscordLogoIcon aria-hidden="true" /></a></nav><div className="farm-editorial-footer-bottom"><span>Copyright © 2024 Farm Community. Built with Farm.</span><a href="/THIRD_PARTY_NOTICES.txt">Third-party notices</a></div>
  </footer>;
}

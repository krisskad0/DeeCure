# DeeCure for security and privacy minded people

[Docs](../../README.md) › [User guide](../README.md) › [Who DeeCure is for](README.md)

**For:** journalists, lawyers, people who handle client data, students of security, and anyone who wants to know their Ubuntu computer is locked down and not leaking.

## What usually goes wrong

- Automatic security updates stopped weeks ago, and nothing said so.
- The firewall is off, or a Docker container quietly opens a port past it.
- An app has camera and microphone access you forgot about.
- A security check list online is long, and you cannot tell which items apply to your machine.

## How DeeCure makes it easy

| You want to… | Open | What you get |
| --- | --- | --- |
| See the whole picture | **Security › Overview** | One checklist: security updates, the firewall, Secure Boot, CPU flaw protection, AppArmor, the screen lock, SSH exposed or not, container ports, and disk encryption, each passed or not, with what to do |
| Check CPU security flaws | **Security › Vulnerabilities** | Each known processor flaw and whether this machine is protected |
| Check how hardened it is | **Security › Hardening** | Kernel protections, home folder permissions, automatic updates, Docker group members (equal to root), and disk encryption, each with advice |
| Check system files | **Security › File Integrity** | A check of installed package files against their packages, and every program that runs with root rights (SUID) |
| Look for warning signs | **Security › Malware Scan** | Quick checks for startup items run from temporary or download folders, cron jobs that download and run scripts, unknown kernel modules, and hidden processes |
| Find what is exposed | **Network › Open Ports**, **Firewall**, and **Containers** | Every listening port and its program, the firewall rules, and container ports that bypass the firewall |
| Know who signs in | **Accounts** | People and groups, sessions, sign-ins per day, and SSH keys |
| See camera, mic, and screen use now | **Privacy › Device Access** | Whether the microphone, camera, screen sharing, and location are in use right now |
| See app permissions | **Privacy › App Permissions** | Which snap and Flatpak apps may use the camera, microphone, location, your files, and the network |
| Check what Ubuntu sends | **Privacy › Privacy Settings** | Settings such as the package popularity survey, and whether it sends your package list |

## Warnings that close the gaps

- **Security updates are waiting**, with a **Fix** that installs them.
- **Automatic security updates stopped working.**
- **The firewall is off** (a warning when something is listening).
- **SSH lets root sign in with a password.**
- **A processor security flaw is not protected.**
- **AppArmor is off**, or **Secure Boot is off**.

**Turning the firewall on** with DeeCure always keeps SSH open first, so you never lock yourself out of a remote machine.

## DeeCure itself is private and careful

- **Nothing about your computer leaves it.** No account, no cloud, no telemetry. The only connection is the check for DeeCure updates, which sends nothing about you and can be turned off.
- **Read-only by default.** About 80% of DeeCure needs no password.
- **Admin steps through a small helper** with a fixed list of actions, its own password prompt for each, input checks, no shell, and an audit log.
- **Checked releases:** every update is verified against the release checksums. Releases carry signed build provenance (Sigstore) and a software bill of materials.
- **Open source** under Apache-2.0, so anyone can check what it does.

## Honest limits

- The malware checks are quick heuristic checks, not an antivirus. DeeCure can point you to a signature scanner, which is optional on Ubuntu.
- A full history of camera and microphone use is not built yet. Today DeeCure shows what is in use right now.

## Your first 10 minutes

1. Open **Security › Overview** and fix every item that is not green.
2. Open **Network › Open Ports**. Close or firewall anything you do not need.
3. Open **Privacy › App Permissions** and remove access you do not use.
4. Open **Security › Hardening** and read the advice.

---

Previous: [Sysadmins and servers](06-sysadmins-and-servers.md) · [All roles](README.md)

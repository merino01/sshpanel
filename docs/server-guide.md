# Server guide

*[Versión en español](guia-servidor.md)*

For the Administrator: how to prepare the Server so that Members can sign in with the app, how to add and remove them, and how to make sure each one can only do what they're meant to.

The commands are for Debian and Ubuntu, and they run on the Server with `sudo`. All the names in this guide are examples: the Server `vps.example.com` (203.0.113.10) and the Members `alice` and `bob`. Replace them with your own.

## How it works, in one sentence

Each Member has their own Linux user on the Server and signs in with a key the app generates on their computer. You authorize that key on the Server, and **the Server is what decides what they can do**: the Profile you give them only configures what the app shows them (see [Modes and security](#modes-and-security)).

## 1. What you need

- A Linux Server with OpenSSH (`sudo apt install openssh-server`), which you can already log in to with `sudo`.
- The `acl` package, for the Shared folder: `sudo apt install acl`.

## 2. Keep the keys where Members can't reach them

By default, OpenSSH reads each user's keys from their own folder (`~/.ssh/authorized_keys`). That won't do here: the Member could change that file with the app itself, lift the limits of their mode and get a shell. The keys will go in `/etc/ssh/authorized_keys/<user>`, which only root can touch.

**Before changing it, copy your own key there**, or you'll lock yourself out:

```sh
sudo install -d -m 755 /etc/ssh/authorized_keys
sudo cp ~/.ssh/authorized_keys /etc/ssh/authorized_keys/$USER
sudo chmod 644 /etc/ssh/authorized_keys/$USER
```

Then, in `/etc/ssh/sshd_config`, leave this line (replace the `AuthorizedKeysFile` one if it already exists):

```
AuthorizedKeysFile /etc/ssh/authorized_keys/%u
```

Check the configuration and apply it **without closing the session you have open**:

```sh
sudo sshd -t && sudo systemctl reload ssh
```

Open another terminal and check that you can still log in before closing the first one.

## 3. Linux users: one per person or one per role

Each Linux user has their home folder and their permissions. There are two ways to share them out:

- **One per person** (`alice`, `bob`…). Each one has their folder, their Trash and their permissions, and the Server shows who did what. It's the tidiest option if there are many of you or you don't know each other.
- **One per role**, for example `team` for everyone and your usual user for administration. There are fewer users to maintain, but whoever shares a user also shares everything else:
  - the same home folder and the same Trash (each one sees, and can restore, what the other deletes);
  - to the Server they're the same person; only its logs tell which key signed in (see step 8).

  What still belongs to each person is their key. Each person has their own line in the user's key file, with their own mode, and you can remove one person's access without touching the others. That way you can have someone in Advanced mode and someone else in Simple mode within the same `team`.

To create a user, whether for a person or a role:

```sh
sudo adduser --disabled-password --gecos "" alice
sudo chmod 750 /home/alice
```

`--disabled-password` means they can only sign in with a key. `chmod 750` stops other users from seeing their home folder.

If you use one user per role, the Shared folder (step 4) is optional: the `team` home folder is already shared.

## 4. Shared folder (optional)

A folder where every Member can upload, change and delete:

```sh
sudo groupadd shared
sudo mkdir -p /srv/shared
sudo chgrp shared /srv/shared
sudo chmod 2770 /srv/shared
sudo setfacl -d -m g:shared:rwx /srv/shared
sudo usermod -aG shared alice
sudo usermod -aG shared bob
```

- `2770`: only the `shared` group gets in, and whatever is created inside still belongs to the group.
- `setfacl -d`: what one person uploads, another can change or delete, including inside new folders.

What gets deleted from the Shared folder goes to the Trash of whoever deleted it, in their home folder. For that, **the Shared folder has to be on the same disk as `/home`**; otherwise, deleting from it fails with a warning and nothing is deleted.

The path (`/srv/shared`) is what you'll put in the "Shared folder" field when creating Profiles.

## 5. Your Administrator Profile

1. Install the app (see the [installation guide](installation-guide.md)).
2. On the welcome screen, click "Do you manage the server? Start by creating your profile".
3. Fill in the Server, your user and, if there is one, the Shared folder. Advanced mode and "Administrator profile" are already ticked. The app connects to the Server to work out its Fingerprint; you don't have to copy it or do anything with it.
4. Save the file and click "Import this profile".
5. The app shows you your public key. Copy it and add this line to your user's key file (step 2), with your key after `restrict,pty` (the Advanced mode options, see [Modes and security](#modes-and-security)):

```sh
sudo nano /etc/ssh/authorized_keys/your-user
```

```
restrict,pty ssh-ed25519 AAAA…your-key… sshpanel
```

If you're still rejected, check that `sudo sshd -T | grep -i authorizedkeysfile` says `/etc/ssh/authorized_keys/%u`: without the change in step 2, OpenSSH doesn't look in that folder.

6. Click "Retry" in the app.

## 6. Adding a Member

1. If they'll have their own user, create it (step 3) and, if needed, add it to the `shared` group (step 4). If you use users per role, it already exists.
2. In the app, "Create profile" (in the top bar): fill in the user (theirs or the role's, like `team`) and their mode, and save the file, for example `alice.sshpanel`. With users per role, the same Profile works for everyone in the role; only each person's key changes.
3. Send them the file. When they import it, the app shows them their public key with a button to copy it, and they send it to you.
4. In "Authorize key", paste their key, write whose it is, choose their mode and copy the line. It looks like this:

```
restrict,command="internal-sftp" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEjEmPlOdEcLaVeNoReAlXxXxXxXxXxXxXxXxXxXxXxX alice-laptop
```

5. Add it to their key file:

```sh
sudo nano /etc/ssh/authorized_keys/alice
sudo chmod 644 /etc/ssh/authorized_keys/alice
```

6. Tell them to click "Retry".

The Profile contains no keys, so you can send it any way you like. The public key isn't secret either.

### Several keys for the same user

A key file can have several lines, one per key. That happens in two cases:

- **The same person on several computers**, for example their laptop and their home computer. Each computer generates its own key, which never leaves it. The person imports the same Profile on both and sends you both keys.
- **Several people with the same user**, if you use users per role (step 3). Each one imports that user's Profile and sends you their key.

In both cases, add one line per key, each with the mode that applies:

```sh
sudo nano /etc/ssh/authorized_keys/team
```

Every key the app generates ends in `sshpanel`. To recognize each line when you need to remove it, **fill in "Whose it is"** in "Authorize key" (for example `alice-laptop`): the line will end in that name instead of `sshpanel`. They end up like this:

```
restrict,pty ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEjEmPlOdEcLaVeNoReAlXxXxXxXxXxXxXxXxXxXxXxX alice-laptop
restrict,command="internal-sftp" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBoBeJeMpLoNoReAlXxXxXxXxXxXxXxXxXxXxXxXxXx bob-home
```

To see which keys a user has, with their Fingerprint and whose each one is:

```sh
sudo ssh-keygen -lf /etc/ssh/authorized_keys/team
```

## Modes and security

The Profile's mode only decides what the app shows. What each Member can really do is decided by the options in front of their key, which is why "Authorize key" adds them:

| Mode | Options | What they can do |
| --- | --- | --- |
| Simple | `restrict,command="internal-sftp"` | Files only (SFTP). No shell or tunnels. |
| Advanced | `restrict,pty` | Files and Terminal. No tunnels or agent forwarding. |

If a Member edits their Profile to give themselves Advanced mode, the app will show them the Terminal, but the Server won't let them open it and the app will explain why.

The same goes for the administrator flag: it only shows "Create profile". Anyone can create Profiles, but a Profile gives access to nothing. Really being an Administrator means being able to edit `/etc/ssh/authorized_keys`.

## 7. Confining Simple mode to its folders

In Simple mode, the app only lets Members use their home folder and the Shared folder. It's a help so they don't get lost, not a limit: with their key, a Member can use any SFTP program and see the rest of the Server wherever Linux gives them read permission. There are two ways to limit it.

### Option A: Linux permissions (recommended)

This is what steps 3 and 4 already do: nobody sees other people's home folders (`chmod 750`) and the Shared folder belongs to the group only. The Member will be able to see system files that are public on any Linux, like those in `/etc` or `/usr`, but not other people's things.

### Option B: confinement with `ChrootDirectory` (advanced)

The Member only sees their folder and the shared one; the rest of the Server doesn't exist for them. It takes more maintenance and has a drawback: **they won't be able to move to the Trash anything in the Shared folder**, because inside the confinement it's mounted separately and is a different file system. They'll get a warning and nothing will be deleted.

To confine `alice`:

```sh
sudo groupadd sftp-only
sudo usermod -aG sftp-only alice

# The root of the confinement has to belong to root, and Alice can't write to it.
sudo chown root:alice /home/alice
sudo chmod 750 /home/alice

# Her files, inside. If she already had files in /home/alice, move them here.
sudo install -d -m 700 -o alice -g alice /home/alice/files

# The Shared folder, mounted inside.
sudo install -d -m 755 /home/alice/shared
echo '/srv/shared /home/alice/shared none bind 0 0' | sudo tee -a /etc/fstab
sudo mount /home/alice/shared
```

And at the end of `/etc/ssh/sshd_config`:

```
Match Group sftp-only
    ChrootDirectory %h
    ForceCommand internal-sftp -d /files
    AllowTcpForwarding no
    X11Forwarding no
```

```sh
sudo sshd -t && sudo systemctl reload ssh
```

In her Profile, the Shared folder is the path **inside the confinement**: `/shared`. Her home folder will be `/files`.

## 8. Removing a Member

Remove their line from their key file (or the whole file) and cut any sessions they have open:

```sh
sudo rm /etc/ssh/authorized_keys/alice
sudo pkill -u alice
```

From that moment they can't sign in, neither with the app nor with any other program. The other Members aren't affected. If you also want to lock the user or delete it along with its files:

```sh
sudo usermod -L alice              # lock it
sudo deluser --remove-home alice   # delete it with its home folder
```

### Removing a single key

If several people share the user, or it's a computer someone has lost, delete only that line:

```sh
sudo nano /etc/ssh/authorized_keys/team
```

That key stops working for signing in from that moment, and the others keep working. If it was connected, the open session continues until it's closed. `pkill -u team` would cut it, but it would also cut the sessions of everyone using `team`; usually it's enough to wait.

To find out which key signed in and when, look for its Fingerprint in the Server log and compare it with the ones from `ssh-keygen -lf`:

```sh
sudo journalctl -u ssh | grep "Accepted publickey for team"
```

## 9. Checking the Fingerprint (optional)

The Fingerprint identifies the Server: the app stores it in the Profile and refuses to connect if it ever doesn't match, in case someone is impersonating it. "Create profile" shows it when it finishes. To check that it's your Server's:

```sh
for f in /etc/ssh/ssh_host_*_key.pub; do ssh-keygen -lf "$f"; done
```

One of the lines, usually the one ending in `(ED25519)`, should have the same Fingerprint, for example `SHA256:EjEmPlOdEhUeLlAnOeSrEaLxXxXxXxXxXxXxXxXxXxX`.

If you reinstall the Server or change its host keys, the Fingerprint changes and everyone's app will stop connecting. You'll have to create new Profiles.

FROM alpine:3.22
# alice tiene sudo con la contraseña `alice-sudo`; bob (`bob-sudo`) no.
RUN apk add --no-cache openssh openssh-sftp-server sudo \
 && for u in alice bob; do \
      adduser -D "$u" \
      && echo "$u:$u-sudo" | chpasswd \
      && mkdir -p "/home/$u/.ssh" \
      && chown -R "$u:$u" "/home/$u/.ssh" \
      && chmod 700 "/home/$u/.ssh"; \
    done \
 && printf '%s\n' \
      'PasswordAuthentication no' \
      'KbdInteractiveAuthentication no' \
      'PermitRootLogin no' \
      'HostKey /etc/ssh/ssh_host_ed25519_key' \
      'AuthorizedKeysFile /etc/ssh/authorized_keys/%u' \
      'Subsystem sftp internal-sftp' \
      'LogLevel VERBOSE' \
      > /etc/ssh/sshd_config \
 && mkdir -m 755 /etc/ssh/authorized_keys \
 && addgroup alice wheel \
 && echo '%wheel ALL=(ALL) ALL' > /etc/sudoers.d/wheel \
 && chmod 440 /etc/sudoers.d/wheel
EXPOSE 22
CMD ["sh", "-c", "ssh-keygen -q -t ed25519 -N '' -f /etc/ssh/ssh_host_ed25519_key && exec /usr/sbin/sshd -D -e"]



The PMRS config file is at `/etc/pmrs/pmrs.conf`. Edit this file to configure PMRS, then run `sudo systemctl restart pmrs` to apply the changes.

Logs are at `/var/log/pmrs/`. PMRS only chowns append permission to these.

The built web dashboard is at `/usr/share/pmrs/dashboard/`. Don't edit it!
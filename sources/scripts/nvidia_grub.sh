#!/bin/bash

if [[ -f /etc/default/grub ]] && ! grep -q "nvidia-drm.modeset=1" /etc/default/grub; then
  sed -i 's/\(GRUB_CMDLINE_LINUX_DEFAULT=["'\'']\)/\1nvidia-drm.modeset=1 /' /etc/default/grub
  grub-mkconfig -o /boot/grub/grub.cfg
fi


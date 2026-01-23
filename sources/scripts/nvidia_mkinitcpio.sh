#!/bin/bash

mods=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)
mkfile="/etc/mkinitcpio.conf"
current_line=$(grep '^MODULES=' "$mkfile" || true)

if [[ $current_line =~ ^MODULES=\"\"$ ]]; then
  sed -i 's/^MODULES=""/MODULES="nvidia nvidia_modeset nvidia_uvm nvidia_drm"/' "$mkfile"
elif [[ $current_line =~ ^MODULES=\(\)$ ]]; then
  sed -i 's/^MODULES=()/MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)/' "$mkfile"
else
  for mod in "${mods[@]}"; do
    grep -qw "$mod" "$mkfile" || sed -i "/^MODULES=(/ s/)/ $mod)/" "$mkfile"
  done
fi


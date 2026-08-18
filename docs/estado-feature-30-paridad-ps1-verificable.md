# Estado archivado - Feature #30: paridad_ps1_verificable
Cerrada: 2026-08-18T02:26:50Z - status=done - Paridad de instaladores verificable sin PowerShell: tests/parity_check.sh compara lo que los dos DECLARAN y falla cuando uno se adelanta. Cierra por trabajo la deuda de once features. El limite (no ejecuta el instalador de Windows) esta escrito en README y verification.md.

---

# Feature #30: paridad_ps1_verificable

Estado: in_progress
Plan: docs/plan-feature-30-paridad-ps1-verificable.md
Spec: docs/spec-feature-30-paridad-ps1-verificable.md

Microservicios:

Evidencia:
- 
- 2026-08-18T02:14:33Z Feature #30 implementada: tests/parity_check.sh compara lo que los dos instaladores DECLARAN (opciones traduciendo kebab a Pascal, superficies, temas de los smokes) sin necesitar PowerShell, con las cinco asimetrias reales declaradas cada una con su razon. Dos razones salieron mal en el primer intento y las encontro la verificacion a mano: --with-postgres es un no-op historico y no la afirmativa de un default, y -CargoTargetDir no tiene que ver con el PATH de rustup.

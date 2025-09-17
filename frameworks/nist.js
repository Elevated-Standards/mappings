import { SecurityFramework, SecurityDomain, SecurityControl } from '../src/core/models.js';

/**
 * NIST Cybersecurity Framework Definition
 */
export function createNISTFramework() {
  const framework = new SecurityFramework(
    'nist-csf',
    'NIST Cybersecurity Framework',
    '1.1',
    'Framework for Improving Critical Infrastructure Cybersecurity'
  );

  // NIST CSF Core Functions
  const domains = [
    new SecurityDomain('ID', 'Identify', 'Develop an organizational understanding to manage cybersecurity risk', 'nist-csf'),
    new SecurityDomain('PR', 'Protect', 'Develop and implement appropriate safeguards to ensure delivery of critical services', 'nist-csf'),
    new SecurityDomain('DE', 'Detect', 'Develop and implement appropriate activities to identify the occurrence of a cybersecurity event', 'nist-csf'),
    new SecurityDomain('RS', 'Respond', 'Develop and implement appropriate activities to take action regarding a detected cybersecurity incident', 'nist-csf'),
    new SecurityDomain('RC', 'Recover', 'Develop and implement appropriate activities to maintain plans for resilience and to restore any capabilities or services', 'nist-csf')
  ];

  domains.forEach(domain => framework.addDomain(domain));

  // NIST CSF Core Controls
  const controls = [
    // Identify
    new SecurityControl(
      'ID.AM-1',
      'Asset Management - Physical Devices and Systems',
      'Physical devices and systems within the organization are inventoried',
      'nist-csf',
      'ID'
    ),
    new SecurityControl(
      'ID.AM-2',
      'Asset Management - Software Platforms and Applications',
      'Software platforms and applications within the organization are inventoried',
      'nist-csf',
      'ID'
    ),
    new SecurityControl(
      'ID.GV-1',
      'Governance - Information Security Policy',
      'Organizational cybersecurity policy is established and communicated',
      'nist-csf',
      'ID'
    ),
    new SecurityControl(
      'ID.RA-1',
      'Risk Assessment - Risk Management Process',
      'Asset vulnerabilities are identified and documented',
      'nist-csf',
      'ID'
    ),

    // Protect
    new SecurityControl(
      'PR.AC-1',
      'Access Control - Identity Management',
      'Identities and credentials are issued, managed, verified, revoked, and audited for authorized devices, users and processes',
      'nist-csf',
      'PR'
    ),
    new SecurityControl(
      'PR.AC-3',
      'Access Control - Remote Access',
      'Remote access is managed',
      'nist-csf',
      'PR'
    ),
    new SecurityControl(
      'PR.DS-1',
      'Data Security - Data-at-rest Protection',
      'Data-at-rest is protected',
      'nist-csf',
      'PR'
    ),
    new SecurityControl(
      'PR.DS-2',
      'Data Security - Data-in-transit Protection',
      'Data-in-transit is protected',
      'nist-csf',
      'PR'
    ),
    new SecurityControl(
      'PR.PT-1',
      'Protective Technology - Audit Logs',
      'Audit/log records are determined, documented, implemented, and reviewed',
      'nist-csf',
      'PR'
    ),

    // Detect
    new SecurityControl(
      'DE.AE-1',
      'Anomalies and Events - Baseline Establishment',
      'A baseline of network operations and expected data flows for users and systems is established and managed',
      'nist-csf',
      'DE'
    ),
    new SecurityControl(
      'DE.CM-1',
      'Security Continuous Monitoring - System Monitoring',
      'The network and physical environment is monitored to detect potential cybersecurity events',
      'nist-csf',
      'DE'
    ),

    // Respond
    new SecurityControl(
      'RS.RP-1',
      'Response Planning - Response Plan',
      'Response plan is executed during or after an incident',
      'nist-csf',
      'RS'
    ),
    new SecurityControl(
      'RS.CO-2',
      'Communications - Incident Reporting',
      'Incidents are reported consistent with established criteria',
      'nist-csf',
      'RS'
    ),

    // Recover
    new SecurityControl(
      'RC.RP-1',
      'Recovery Planning - Recovery Plan',
      'Recovery plan is executed during or after a cybersecurity incident',
      'nist-csf',
      'RC'
    ),
    new SecurityControl(
      'RC.IM-1',
      'Improvements - Lessons Learned',
      'Recovery plans incorporate lessons learned',
      'nist-csf',
      'RC'
    )
  ];

  controls.forEach(control => {
    control.addTag('nist');
    control.addTag('cybersecurity');
    control.addTag('framework');
    
    // Set control types
    if (control.id.includes('PR.AC') || control.id.includes('PR.DS') || control.id.includes('DE.CM')) {
      control.controlType = 'technical';
    } else if (control.id.includes('PR.PT')) {
      control.controlType = 'technical';
    }
    
    framework.addControl(control);
  });

  return framework;
}
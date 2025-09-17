import { SecurityFramework, SecurityDomain, SecurityControl } from '../src/core/models.js';

/**
 * ISO 27001 Framework Definition
 * Information Security Management System
 */
export function createISO27001Framework() {
  const framework = new SecurityFramework(
    'iso27001',
    'ISO 27001',
    '2022',
    'Information Security Management System - Requirements for establishing, implementing, maintaining and continually improving an information security management system'
  );

  // ISO 27001 Annex A Control Domains
  const domains = [
    new SecurityDomain('A.5', 'Information Security Policies', 'Organizational information security', 'iso27001'),
    new SecurityDomain('A.6', 'Organization of Information Security', 'Internal organization and mobile devices', 'iso27001'),
    new SecurityDomain('A.7', 'Human Resource Security', 'Personnel security controls', 'iso27001'),
    new SecurityDomain('A.8', 'Asset Management', 'Asset responsibility and information classification', 'iso27001'),
    new SecurityDomain('A.9', 'Access Control', 'Business requirements for access control', 'iso27001'),
    new SecurityDomain('A.10', 'Cryptography', 'Cryptographic controls', 'iso27001'),
    new SecurityDomain('A.11', 'Physical and Environmental Security', 'Secure areas and equipment protection', 'iso27001'),
    new SecurityDomain('A.12', 'Operations Security', 'Operational procedures and responsibilities', 'iso27001'),
    new SecurityDomain('A.13', 'Communications Security', 'Network security management', 'iso27001'),
    new SecurityDomain('A.14', 'System Acquisition, Development and Maintenance', 'Security in development and support processes', 'iso27001'),
    new SecurityDomain('A.15', 'Supplier Relationships', 'Information security in supplier relationships', 'iso27001'),
    new SecurityDomain('A.16', 'Information Security Incident Management', 'Management of information security incidents', 'iso27001'),
    new SecurityDomain('A.17', 'Information Security Aspects of Business Continuity Management', 'Business continuity planning', 'iso27001'),
    new SecurityDomain('A.18', 'Compliance', 'Compliance with legal and contractual requirements', 'iso27001')
  ];

  domains.forEach(domain => framework.addDomain(domain));

  // Key ISO 27001 Controls
  const controls = [
    // Information Security Policies
    new SecurityControl(
      'A.5.1.1',
      'Information Security Policy',
      'An information security policy shall be defined, approved by management, published and communicated to employees and relevant external parties',
      'iso27001',
      'A.5'
    ),
    new SecurityControl(
      'A.5.1.2',
      'Review of Information Security Policy',
      'The information security policy shall be reviewed at planned intervals or if significant changes occur',
      'iso27001',
      'A.5'
    ),

    // Organization of Information Security
    new SecurityControl(
      'A.6.1.1',
      'Information Security Roles and Responsibilities',
      'All information security responsibilities shall be defined and allocated',
      'iso27001',
      'A.6'
    ),
    new SecurityControl(
      'A.6.2.1',
      'Mobile Device Policy',
      'A policy and supporting security measures shall be adopted to manage the risks introduced by using mobile devices',
      'iso27001',
      'A.6'
    ),

    // Human Resource Security
    new SecurityControl(
      'A.7.1.1',
      'Screening',
      'Background verification checks on all candidates for employment shall be carried out in accordance with relevant laws, regulations and ethics',
      'iso27001',
      'A.7'
    ),
    new SecurityControl(
      'A.7.2.2',
      'Information Security Awareness, Education and Training',
      'All employees of the organization and, where relevant, contractors shall receive appropriate awareness education and training',
      'iso27001',
      'A.7'
    ),

    // Asset Management
    new SecurityControl(
      'A.8.1.1',
      'Inventory of Assets',
      'Assets associated with information and information processing facilities shall be identified',
      'iso27001',
      'A.8'
    ),
    new SecurityControl(
      'A.8.2.1',
      'Classification of Information',
      'Information shall be classified in terms of legal requirements, value, criticality and sensitivity',
      'iso27001',
      'A.8'
    ),

    // Access Control
    new SecurityControl(
      'A.9.1.1',
      'Access Control Policy',
      'An access control policy shall be established, documented and reviewed based on business and information security requirements',
      'iso27001',
      'A.9'
    ),
    new SecurityControl(
      'A.9.2.1',
      'User Registration and De-registration',
      'A formal user registration and de-registration process shall be implemented to enable assignment of access rights',
      'iso27001',
      'A.9'
    ),

    // Cryptography
    new SecurityControl(
      'A.10.1.1',
      'Policy on the Use of Cryptographic Controls',
      'A policy on the use of cryptographic controls for protection of information shall be developed and implemented',
      'iso27001',
      'A.10'
    ),

    // Physical and Environmental Security
    new SecurityControl(
      'A.11.1.1',
      'Physical Security Perimeter',
      'Security perimeters shall be defined and used to protect areas that contain either sensitive or critical information',
      'iso27001',
      'A.11'
    ),

    // Operations Security
    new SecurityControl(
      'A.12.1.2',
      'Change Management',
      'Changes to the organization, business processes, information processing facilities and systems shall be controlled',
      'iso27001',
      'A.12'
    ),
    new SecurityControl(
      'A.12.6.1',
      'Management of Technical Vulnerabilities',
      'Information about technical vulnerabilities of information systems being used shall be obtained in a timely fashion',
      'iso27001',
      'A.12'
    )
  ];

  controls.forEach(control => {
    // Add common tags
    control.addTag('iso27001');
    control.addTag('isms');
    control.addTag('information-security');
    
    // Set risk levels based on control criticality
    if (control.id.includes('A.9') || control.id.includes('A.10')) {
      control.riskLevel = 'high';
    } else if (control.id.includes('A.5') || control.id.includes('A.12.6')) {
      control.riskLevel = 'critical';
    }
    
    framework.addControl(control);
  });

  return framework;
}
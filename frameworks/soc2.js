import { SecurityFramework, SecurityDomain, SecurityControl } from '../src/core/models.js';

/**
 * SOC 2 Framework Definition
 * System and Organization Controls 2
 */
export function createSOC2Framework() {
  const framework = new SecurityFramework(
    'soc2',
    'SOC 2',
    '2017',
    'System and Organization Controls 2 - Trust Services Criteria for Security, Availability, Processing Integrity, Confidentiality, and Privacy'
  );

  // SOC 2 Trust Services Categories (Domains)
  const domains = [
    new SecurityDomain('security', 'Security', 'The system is protected against unauthorized access', 'soc2'),
    new SecurityDomain('availability', 'Availability', 'The system is available for operation and use', 'soc2'),
    new SecurityDomain('processing_integrity', 'Processing Integrity', 'System processing is complete, valid, accurate, timely, and authorized', 'soc2'),
    new SecurityDomain('confidentiality', 'Confidentiality', 'Information designated as confidential is protected', 'soc2'),
    new SecurityDomain('privacy', 'Privacy', 'Personal information is collected, used, retained, disclosed, and disposed of in conformity with commitments', 'soc2')
  ];

  domains.forEach(domain => framework.addDomain(domain));

  // SOC 2 Security Controls (Common Criteria)
  const securityControls = [
    new SecurityControl(
      'CC1.1',
      'Control Environment - Integrity and Ethical Values',
      'The entity demonstrates a commitment to integrity and ethical values',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC1.2', 
      'Control Environment - Board Independence',
      'The board of directors demonstrates independence from management and exercises oversight',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC2.1',
      'Communication and Information - Internal Communication',
      'The entity obtains or generates and uses relevant, quality information to support the functioning of internal control',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC3.1',
      'Risk Assessment - Objectives',
      'The entity specifies objectives with sufficient clarity to enable the identification and assessment of risks',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC4.1',
      'Monitoring Activities - Ongoing Monitoring',
      'The entity selects, develops, and performs ongoing and/or separate evaluations',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC5.1',
      'Control Activities - Selection and Development',
      'The entity selects and develops control activities that contribute to the mitigation of risks',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC6.1',
      'Logical and Physical Access Controls - Access Control',
      'The entity implements logical access security software, infrastructure, and architectures over protected information assets',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC6.2',
      'Logical and Physical Access Controls - Authentication',
      'Prior to issuing system credentials and granting system access, the entity registers and authorizes new internal and external users',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC6.7',
      'System Operations - Data Transmission',
      'The entity restricts the transmission of data and software to defined system users',
      'soc2',
      'security'
    ),
    new SecurityControl(
      'CC7.1',
      'System Operations - System Monitoring',
      'The entity monitors the system and various communications channels for security events',
      'soc2',
      'security'
    )
  ];

  // Availability Controls
  const availabilityControls = [
    new SecurityControl(
      'A1.1',
      'Availability - System Capacity',
      'The entity maintains system capacity consistent with system processing requirements',
      'soc2',
      'availability'
    ),
    new SecurityControl(
      'A1.2',
      'Availability - Environmental Protection',
      'The entity authorizes, designs, develops or acquires, implements, operates, approves, maintains, and monitors environmental protections',
      'soc2',
      'availability'
    )
  ];

  const allControls = [...securityControls, ...availabilityControls];
  allControls.forEach(control => {
    // Add common tags
    control.addTag('soc2');
    control.addTag('audit');
    control.addTag('compliance');
    
    // Set control types
    if (control.id.startsWith('CC6') || control.id.startsWith('CC7')) {
      control.controlType = 'technical';
    }
    
    framework.addControl(control);
  });

  return framework;
}